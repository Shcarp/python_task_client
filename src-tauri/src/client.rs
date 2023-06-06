use std::{
    fmt::Debug,
    net::TcpStream,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH}, mem,
};
use log::{info, error};
use serde_json::Value;
use tokio::sync::{Mutex as AsyncMutex, mpsc::{Sender}};
use anyhow::{Result};
use dashmap::DashMap;
use proto::{
    message::{Body, DataType, Push, Request, Response, Status as MessageState},
    MessageType,
};
use protobuf::{Message, SpecialFields};
use std::any::Any;
use tauri::{Runtime, Window};
use thiserror::Error;
use uuid::Uuid;
use websocket::{ClientBuilder, sync::{Reader, Writer}};

use crate::promise::{Promise, PromiseResult};

#[derive(Error, Debug, serde::Serialize)]
pub enum ConnError {
    #[error("connect error: {0}")]
    ConnectError(String),
    #[error("send error: {0}")]
    SendError(String),
    #[error("lock error: {0}")]
    LockError(String),
}

const CLIENT_IDENTIFICATION: &str = "CLIENT_IDENTIFICATION";

pub fn uniform_event_name(name: &str) -> String {
    format!("{}::{}", CLIENT_IDENTIFICATION, name)
}

macro_rules! handle_message {
    ($payload:expr, $index:expr, $message_type:ident, $clients:expr, $address:expr) => {
        match $message_type::parse_from_bytes(&$payload[$index..]) {
            Ok(data) => {
                let clients = $clients
                    .read()
                    .map_err(|error| ConnError::LockError(error.to_string()))
                    .unwrap();
                for client in clients.iter() {
                    if client.address == $address {
                        $message_type::handle_message(client.clone(), Box::new(data.clone()));
                    }
                }
            }
            Err(error) => {
                error!("parse data error: {}", error);
            }
        }
    };
}

macro_rules! distribute {
    ($client:expr, $data:expr, $type:ident, $handle:ident) => {
        match $data.downcast::<$type>() {
            Ok(res) => {
                let fns = move || {
                    tokio::spawn(async move {
                        $client.$handle(*res).await;
                    });
                };
                tokio::task::spawn_blocking(fns);
            }
            Err(_) => {
                println!("ERROR")
            }
        }
    };
}

pub trait MessageBody {
    fn from_serialize(data: Value) -> Body;
}

impl MessageBody for Body {
    fn from_serialize(data: Value) -> Body {
        let type_ = match data {
            Value::String(_) => DataType::String,
            Value::Number(_) => DataType::Number,
            Value::Bool(_) => DataType::Bool,
            Value::Array(_) => DataType::Array,
            Value::Object(_) => DataType::Object,
            Value::Null => DataType::Object,
        };
        Body {
            type_: Some(type_.into()),
            value: serde_json::to_string(&data).unwrap(),
            special_fields: SpecialFields::default(),
        }
    }
}

trait MessageHandler {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>);
}

impl MessageHandler for Push {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>) {
        distribute!(client, data, Push, handle_push);
    }
}

impl MessageHandler for Request {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>) {
        distribute!(client, data, Request, handle_request);
    }
}

impl MessageHandler for Response {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>) {
        distribute!(client, data, Response, handle_response);
    }
}

#[derive(Clone)]
pub struct Conn {
    pub address: String,
    pub reader: Arc<AsyncMutex<Reader<TcpStream>>>,
    pub writer: Arc<AsyncMutex<Writer<TcpStream>>>,
}

#[derive(Default)]
pub struct ClientManage<R: Runtime> {
    clients: Arc<RwLock<Vec<Arc<WClient<R>>>>>,
    conns: Arc<DashMap<String, Conn>>,
}

impl<R: Runtime> ClientManage<R> {
    pub fn new() -> ClientManage<R> {
        ClientManage {
            clients: Arc::new(RwLock::new(vec![])),
            conns: Arc::new(DashMap::new()),
        }
    }

    pub fn add_client(&mut self, win: Window<R>, address: &str) -> Result<String> {
        // 判断address是否已经存在, 如果已存在则用存在的conn
        if let Some(conn) = self.conns.get(address) {
            let client = Arc::new(WClient::new(win, address.to_string(), conn.clone()));
            let client_id = client.client_id.clone();
            self.clients
                .write()
                .map_err(|error| ConnError::LockError(error.to_string()))?
                .push(client);
            Ok(client_id)
        } else {
            let res = ClientBuilder::new(address)?.connect_insecure();
            match res {
                Ok(conn) => {
                    match conn.split() {
                        Ok((reader, writer)) => {
                            let conn = Conn {
                                address: address.to_string(),
                                reader: Arc::new(AsyncMutex::new(reader)),
                                writer: Arc::new(AsyncMutex::new(writer)),
                            };
                            let client = Arc::new(WClient::new(win, address.to_string(), conn.clone()));
                            let client_id = client.client_id.clone();

                            self.clients
                                .write()
                                .map_err(|error| ConnError::LockError(error.to_string()))?
                                .push(client);

                            self.conns.insert(address.to_string(), conn.clone());

                            // 启动读取线程
                            let clients = self.clients.clone();
                            let conns = self.conns.clone();
                            let address = address.to_string();
                            tokio::spawn(
                                async move { 
                                    read_loop(address, conn, clients, conns).await 
                                }
                            );
                            Ok(client_id)
                        },
                        Err(error) => {
                            win.emit(&uniform_event_name("error"), Some(&error.to_string()))?;
                            Err(ConnError::ConnectError(error.to_string()).into())
                        },
                    }
                }
                Err(error) => {
                    win.emit(&uniform_event_name("error"), Some(&error.to_string()))?;
                    Err(ConnError::ConnectError(error.to_string()).into())
                }
            }
        }
    }

    pub fn get_client(&self, client_id: String) -> Option<Arc<WClient<R>>> {
        self.clients
            .read()
            .map_err(|error| ConnError::LockError(error.to_string()))
            .unwrap()
            .iter()
            .find(|client| client.client_id == client_id)
            .map(|client| client.clone())
    }

    pub fn remove_client(&mut self, win: &Window<R>, client_id: String) -> Result<()> {
        self.clients
            .write()
            .unwrap()
            .retain(|client| client.client_id != client_id);
        win.emit(&uniform_event_name("close"), "close")?;
        Ok(())
    }

    pub async fn close_all(&mut self) -> Result<()> {
        for client in self
            .clients
            .write()
            .map_err(|error| ConnError::LockError(error.to_string()))?
            .iter()
        {
            client.window.emit(&uniform_event_name("close"), "close")?;
            let reader = client.conn.reader.lock().await;
            let writer = client.conn.writer.lock().await;
            reader.shutdown()?;
            writer.shutdown()?;
        }
        self.clients = Arc::new(RwLock::new(vec![]));
        Ok(())
    }
}

pub struct WClient<R: Runtime> {
    pub client_id: String,
    pub window: Window<R>,
    pub address: String,
    pub conn: Conn,
    pub sequences: DashMap<String, Sender<PromiseResult<Body>>>,
}

impl<R: Runtime> WClient<R> {
    pub fn new(
        win: Window<R>,
        address: String,
        conn: Conn,
    ) -> WClient<R> {
        let client = WClient {
            client_id: Uuid::new_v4().to_string(),
            window: win,
            address: address,
            conn: conn,
            sequences: DashMap::new(),
        };
        client
    }

    pub async fn request(&self, url: String, data: Body) -> Body {
        let (promise, sender) = Promise::<Body>::new();
        let sequence = Uuid::new_v4().to_string();

        let mut request = Request::new();

        request.sequence = sequence.clone();
        request.type_ = "request".to_string();
        request.url = url.clone();
        request.data = Some(data).into();
        request.sendTime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32;

        // 在request 序列化数据前加上请求标识
        match request.write_to_bytes() {
            Ok(mut data) => {
                data.insert(0, MessageType::REQUEST.into());

                match self.send(&data).await {
                    Ok(_) => {
                         match self.window.emit(&uniform_event_name("send"), "success") {
                            Ok(_) => {},
                            Err(_) => {
                                error!("emit event error");
                            },
                        };
                    }
                    Err(error) => {
                        error!("request error: {:?}", error);
                        sender.send(PromiseResult::Rejected).await.unwrap();
                        return Body::default();
                    }
                }

                self.sequences.insert(sequence.clone(), sender);

                let res = promise.await;
                res
            }
            Err(_) => proto::message::Body::from_serialize(Value::String("data error".to_string())),
        }
    }

    pub async fn handle_response(&self, response: Response) {
        let sequence = response.sequence.clone();
        if let Some(sender) = self.sequences.remove(&sequence) {
            let sender = sender.1;
            match response.status {
                Some(status) => {
                    if status != MessageState::OK.into() {
                        Promise::reject(sender).await;
                        return;
                    }
                    match response.data.0 {
                        Some(data) => {
                            Promise::resolve(sender, *data).await;
                        }
                        _ => {
                            Promise::resolve(sender, Body::default()).await;
                        }
                    }
                }
                None => {
                    Promise::reject(sender).await;
                },
            }
        }
    }

    pub async fn handle_push(&self, push: Push) {
        println!("push");
    }

    pub async fn handle_request(&self, request: Request) {
        println!("request: {:?}", request);
    }

    pub async fn send(&self, data: &[u8]) -> Result<()> {
        println!("send");
        let mut writer = self.conn.writer.lock().await;
        match writer.send_message(&websocket::message::Message::binary(data)) {
            Ok(_) => {
                // println!("send success")
            }
            Err(error) => {
                drop(writer);
                match reconnect(self.address.clone(), self.conn.clone()).await {
                    Ok(_) => {
                        let mut writer = self.conn.writer.lock().await;
                        writer.send_message(&websocket::message::Message::binary(data))?;
                    }
                    Err(_) => {
                        let err = ConnError::SendError(error.to_string());
                        self.window
                            .emit(&uniform_event_name("error"), err.to_string())?;
                    }
                };
            }
        }
        Ok(())
    }
}

impl<R: Runtime> Debug for WClient<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WClient")
            .field("client_id", &self.client_id)
            .field("address", &self.address)
            .field("window", &self.window.label())
            .field("sequences", &self.sequences)
            .finish()
    }
}

async fn read_loop<R: Runtime>(
    address: String,
    conn: Conn,
    clients: Arc<RwLock<Vec<Arc<WClient<R>>>>>,
    conns:  Arc<DashMap<String, Conn>>,
) {
    loop {
        let mut reader = conn.reader.lock().await;
        let close = || {
            // 从clients中删除 address 对应的client
            clients
                .write()
                .unwrap()
                .retain(|client| client.address != address);
        };

        match reader.recv_message() {
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
                            }
                            MessageType::REQUEST => {
                                println!("recv: request");
                                handle_message!(payload, index, Request, clients, address);
                            }
                            MessageType::RESPONSE => {
                                println!("recv: response");
                                handle_message!(payload, index, Response, clients, address);
                            }
                            MessageType::OTHER => {
                                println!("recv: binary {:?}", payload);
                            }
                        };
                    }
                    websocket::message::OwnedMessage::Close(_) => {
                        info!("recv: close");
                        close();
                        reader.shutdown().unwrap();
                    }
                    websocket::message::OwnedMessage::Ping(payload) => {
                        println!("recv: ping");
                        let mut writer = conn.writer.lock().await;
                        writer.send_message(&websocket::message::Message::pong(payload)).unwrap();
                    }
                    websocket::message::OwnedMessage::Pong(_) => {
                        println!("recv: pong");
                    }
                }
            }

            Err(_) => {
                drop(reader);
                match reconnect(address.clone(), conn.clone()).await {
                    Ok(_) => continue,
                    Err(_) => {
                        close();
                        conns.remove(&address);
                        break;
                    }
                }
            } 
        }
        drop(reader)
    }
}

async fn reconnect(address: String, conn: Conn) -> Result<(), ()> {
    // 重试10次
    let mut count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let res = ClientBuilder::new(address.as_str())
            .unwrap()
            .connect_insecure();
        match res {
            Ok(new_conn) => {
                info!("reconnect success {:#?}", new_conn.local_addr());
                match new_conn.split() {
                    Ok((reader, writer)) => {
                        let mut reader_guard = conn.reader.lock().await;
                        let mut writer_guard = conn.writer.lock().await;
                        let old_reader = mem::replace(&mut *reader_guard, reader);
                        let old_writer = mem::replace(&mut *writer_guard, writer);
                        info!("reconnect success");
                        drop(old_reader);
                        drop(old_writer);
                        return Ok(());
                    },
                    Err(_) =>{
                        return Err(());
                    },
                }
            }
            Err(error) => {
                error!("reconnect error: {}", error);
                count += 1;
                if count > 10 {
                    return Err(());
                }
            }
        }
    }
}
