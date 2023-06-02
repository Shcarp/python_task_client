use std::{
    net::TcpStream,
    sync::{Arc, Mutex, RwLock}, fmt::{Debug},
};

use thiserror::Error;
use std::any::Any;
use protobuf::{ Message };
use dashmap::DashMap;
use anyhow::{Result};
use proto::{message::{Push, Request, Response}, MessageType};
use tauri::{Window, Runtime};
use uuid::Uuid;
use websocket::{client::sync::Client, ClientBuilder};

#[derive(Error, Debug, serde::Serialize)]
pub enum ConnError {
    #[error("connect error: {0}")]
    ConnectError(String),
    #[error("send error: {0}")]
    SendError(String),
    #[error("recv error")]
    RecvError,
    #[error("lock error: {0}")]
    LockError(String)
}

const CLIENT_IDENTIFICATION: &str = "CLIENT_IDENTIFICATION";

pub fn uniform_event_name(name: &str) -> String {
    format!("{}::{}", CLIENT_IDENTIFICATION, name)
}

macro_rules! handle_message  {
    ($payload:expr, $index:expr, $message_type:ident, $clients:expr, $address:expr) => {
        match $message_type::parse_from_bytes(&$payload[$index..]) {
            Ok(data) => {
                let clients = $clients.read().map_err(|error| ConnError::LockError(error.to_string())).unwrap();
                for client in clients.iter() {
                    if client.address == $address {
                        $message_type::handle_message(client.clone(), Box::new(data.clone()));
                    }
                }
            },
            Err(_) => { 
                println!("ERROR")
            },
        }
    };
}

trait MessageHandler {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>);
}

impl MessageHandler for Push {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>) {
        match data.downcast::<Push>() {
            Ok(push) => {
                client.handle_push(*push);
            },
            Err(_) => {
                println!("ERROR")
            },
        }
    }
}

impl MessageHandler for Request {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>) {
        match data.downcast::<Request>() {
            Ok(request) => {
                client.handle_request(*request);
            },
            Err(_) => {
                println!("ERROR")
            },
        }
    }
}

impl MessageHandler for Response {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>) {
       match data.downcast::<Response>() {
           Ok(response) => {
               client.handle_response(*response);
           },
           Err(_) => {
               println!("ERROR")
           },
       }
    }
}

#[derive(Default)]
pub struct ClientManage<R: Runtime> {
    clients: Arc<RwLock<Vec<Arc<WClient<R>>>>>,
    conn_pool: Arc<DashMap<String, Arc<Mutex<Client<TcpStream>>>>>,
}

impl<R: Runtime> ClientManage<R> {
    pub fn new() -> ClientManage<R> {
        ClientManage {
            clients: Arc::new(RwLock::new(vec![])),
            conn_pool: Arc::new(DashMap::new()),
        }
    }

    pub fn add_client(&mut self, win: Window<R>, address: &str) -> Result<String> {
        // 判断address是否已经存在, 如果已存在则用存在的conn
        if let Some(conn) = self.conn_pool.get(address) {
            let client = Arc::new(WClient::new(win, address.to_string(), conn.clone()));
            let client_id = client.client_id.clone();
            self.clients.write().map_err(|error| ConnError::LockError(error.to_string()))?.push(client);
            Ok(client_id)
        }  else {
            let res = ClientBuilder::new(address)?.connect_insecure();
            match res {
                Ok(conn) => {
                    let conn = Arc::new(Mutex::new(conn));
                    let conn_clone = conn.clone();
                    let client = Arc::new(WClient::new(win, address.to_string(), conn.clone()));
                    let client_id = client.client_id.clone();

                    self.clients.write().map_err(|error| ConnError::LockError(error.to_string()))?.push(client);
                    self.conn_pool.insert(address.to_string(), conn);

                    // 启动读取线程
                    let clients = self.clients.clone();
                    let conns = self.conn_pool.clone();
                    let address = address.to_string();
                    std::thread::spawn(move || {
                        read_loop(address, conn_clone, clients, conns);
                    });

                    Ok(client_id)
                }
                Err(error) => {
                    win.emit(&uniform_event_name("error"), Some(&error.to_string()))?;
                    Err(ConnError::ConnectError(error.to_string()).into())
                }
            }
        }
    }

    pub fn remove_client(&mut self, win: &Window, address: &str) -> Result<()> {
        self.clients.write().unwrap().retain(|client| client.address != address);
        win.emit(&uniform_event_name("close"), "close")?;
        Ok(())
    }

    pub fn close_all(&mut self) -> Result<()> {
        for client in self.clients.write().map_err(|error| ConnError::LockError(error.to_string()))?.iter() {
            client.window.emit(&uniform_event_name("close"), "close")?;
            let conn = client.conn.lock().map_err(|error| ConnError::LockError(error.to_string()))?;
            conn.shutdown()?;
        }
        self.clients = Arc::new(RwLock::new(vec![]));
        Ok(())
    }
}

pub struct WClient<R: Runtime> {
    pub client_id: String,
    pub window: Window<R>,
    pub address: String,
    pub conn: Arc<Mutex<Client<TcpStream>>>,
    pub sequences: Vec<u64>,
}

impl<R: Runtime> WClient<R> {
    pub fn new(win: Window<R>, address: String, conn: Arc<Mutex<Client<TcpStream>>>) -> WClient<R> {
        let client = WClient {
            client_id: Uuid::new_v4().to_string(),
            window: win,
            address: address,
            conn: conn,
            sequences: vec![],
        };
        client
    }

    pub fn handle_push(&self, push: Push) {
        println!("push: {:?}", push);
    }

    pub fn handle_request(&self, request: Request) {
        println!("request: {:?}", request);
    }

    pub fn handle_response(&self, response: Response) {
        println!("response: {:?}", response);
    }

    pub fn send(&self, data: Vec<u8>) -> Result<()> {
        let mut conn = self.conn.lock().map_err(|error| ConnError::LockError(error.to_string()))?;

        match conn.send_message(&websocket::message::Message::binary(data.clone())) {
            Ok(_) => {},
            Err(error) => {
                match reconnect(self.address.clone(), self.conn.clone()) {
                    Ok(_) => {
                        conn = self.conn.lock().map_err(|error| ConnError::LockError(error.to_string()))?;
                        conn.send_message(&websocket::message::Message::binary(data.clone()))?;
                    },
                    Err(_) => {
                        let err = ConnError::SendError(error.to_string());
                        self.window.emit(&uniform_event_name("error"), err.to_string())?;
                    },
                };
            }
        }
        Ok(())
    }
}

impl<R: Runtime> Drop for WClient<R> {
    fn drop(&mut self) {
        self.address = "".to_string();
        self.window.emit(&uniform_event_name("close"), "close").unwrap();
        drop(&self.window);
        drop(&self.conn);
        drop(&self.sequences);
    }
}

impl <R: Runtime> Debug for WClient<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WClient").field("client_id", &self.client_id).field("address", &self.address).field("window", &self.window.label()).field("sequences", &self.sequences).finish()
    }
}

fn read_loop<R: Runtime>(
    address: String, 
    conn: Arc<Mutex<Client<TcpStream>>>, 
    clients: Arc<RwLock<Vec<Arc<WClient<R>>>>>, 
    conns: Arc<DashMap<String, Arc<Mutex<Client<TcpStream>>>>>
) {
    loop {
        let mut mconn = conn.lock().unwrap();

        let close = || {
            // 从clients中删除 address 对应的client
            clients.write().unwrap().retain(|client| client.address != address);
        };

        match mconn.recv_message() {
            Ok(response) => {
                match response {
                    websocket::message::OwnedMessage::Text(text) => {
                        println!("recv: {}", text);
                    }
                    websocket::message::OwnedMessage::Binary(payload) => {
                        let mut index = 0;

                        // 读取第一个字节
                        let first_byte = payload[index];
                        // 转换为MessageType
                        let first_byte = first_byte as u8;
                        index += 1;
                        match MessageType::from_u8(first_byte) {
                            MessageType::PUSH => {
                                handle_message!(payload, index, Push, clients, address);
                            },
                            MessageType::REQUEST => {
                                // handle_message!(payload, index, Request, handle_request, address);
                                handle_message!(payload, index, Request, clients, address);
                            },
                            MessageType::RESPONSE => {
                                // handle_message!(payload, index, Response, handle_response, address);
                                handle_message!(payload, index, Response, clients, address);
                            },
                            MessageType::OTHER => {
                                println!("recv: binary {:?}", payload);
                            },
                        };
                        // println!("recv: binary {:?}", push);
                    }
                    websocket::message::OwnedMessage::Close(_) => {
                        println!("recv: close");
                        close();
                        mconn.shutdown().unwrap();
                    }
                    websocket::message::OwnedMessage::Ping(payload) => {
                        println!("recv: ping");
                        mconn.send_message(&websocket::message::Message::pong(payload)).unwrap();
                    }
                    websocket::message::OwnedMessage::Pong(_) => {
                        println!("recv: pong");
                        // conn.send_message(&websocket::message::Message::pong(payload)).unwrap();
                    }
                }
            },
            Err(_) => {
                match reconnect(address.clone(), conn.clone()) {
                    Ok(_) => continue,
                    Err(_) => {
                        close();
                        conns.remove(&address);
                        break;
                    }
                } 
            },
        }
    }
}

fn reconnect(address: String, conn: Arc<Mutex<Client<TcpStream>>>) -> Result<(), ()> {
    // 重试10次
    let mut count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let res = ClientBuilder::new(address.as_str()).unwrap().connect_insecure();
        match res {
            Ok(new_conn) => {
                let mut conn = conn.lock().unwrap();
                *conn = new_conn;
                return Ok(());
            }
            Err(error) => {
                println!("reconnect error: {}", error);
                count += 1;
                if count > 10 {
                    return Err(());
                }
            }
        }
    }
}
