use std::{
    fmt::Debug,
    net::TcpStream,
    sync::{Arc, Mutex, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use tokio::sync::Mutex as AsyncMutex;

use anyhow::Result;
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
use websocket::{client::sync::Client, ClientBuilder};

use crate::promise::Promise;

#[derive(Error, Debug, serde::Serialize)]
pub enum ConnError {
    #[error("connect error: {0}")]
    ConnectError(String),
    #[error("send error: {0}")]
    SendError(String),
    #[error("recv error")]
    RecvError,
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
        match data.downcast::<Push>() {
            Ok(push) => {
                tokio::spawn(async move {
                    client.handle_push(*push).await;
                });
            }
            Err(_) => {
                println!("ERROR")
            }
        }
    }
}

impl MessageHandler for Request {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>) {
        match data.downcast::<Request>() {
            Ok(request) => {
                tokio::spawn(async move {
                    client.handle_request(*request).await;
                });
            }
            Err(_) => {
                println!("ERROR")
            }
        }
    }
}

impl MessageHandler for Response {
    fn handle_message<R: Runtime>(client: Arc<WClient<R>>, data: Box<dyn Any>) {
        match data.downcast::<Response>() {
            Ok(response) => {
                tokio::spawn(async move {
                    client.handle_response(*response).await;
                });
            }
            Err(_) => {
                println!("ERROR")
            }
        }
    }
}

#[derive(Default)]
pub struct ClientManage<R: Runtime> {
    clients: Arc<RwLock<Vec<Arc<WClient<R>>>>>,
    conn_pool: Arc<DashMap<String, Arc<AsyncMutex<Client<TcpStream>>>>>,
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
            self.clients
                .write()
                .map_err(|error| ConnError::LockError(error.to_string()))?
                .push(client);
            Ok(client_id)
        } else {
            let res = ClientBuilder::new(address)?.connect_insecure();
            match res {
                Ok(conn) => {
                    let conn = Arc::new(AsyncMutex::new(conn));
                    let conn_clone = conn.clone();
                    let client = Arc::new(WClient::new(win, address.to_string(), conn.clone()));
                    let client_id = client.client_id.clone();

                    let clients = self.clients
                        .write()
                        .map_err(|error| ConnError::LockError(error.to_string()))?
                        .push(client);
                    self.conn_pool.insert(address.to_string(), conn);

                    // 释放锁
                    drop(clients);

                    // 启动读取线程
                    let clients = self.clients.clone();
                    let conns = self.conn_pool.clone();
                    let address = address.to_string();

                    tokio::spawn(
                        async move { println!("{}", 11); read_loop(address, conn_clone, clients, conns).await },
                    );

                    Ok(client_id)
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
            let conn = client.conn.lock().await;
            //
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
    pub conn: Arc<AsyncMutex<Client<TcpStream>>>,
    pub sequences: DashMap<String, Arc<AsyncMutex<Promise<Body>>>>,
}

impl<R: Runtime> WClient<R> {
    pub fn new(
        win: Window<R>,
        address: String,
        conn: Arc<AsyncMutex<Client<TcpStream>>>,
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
        let promise = Arc::new(AsyncMutex::new(Promise::<Body>::new()));

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
                println!("request: {:?}", data);
                data.insert(0, MessageType::REQUEST.into());
                println!("request: {:?}", data);

                self.send(&data).await;

                self.sequences.insert(sequence.clone(), promise.clone());

                let mut res = promise.lock().await; 
                if let Some(body) = res.value() {
                    body
                } else {
                    Body::default()
                }
            }
            Err(_) => Body::default(),
        }
    }

    pub async fn handle_push(&self, push: Push) {
        println!("push: {:?}", push);
    }

    pub async fn handle_request(&self, request: Request) {
        println!("request: {:?}", request);
    }

    pub async fn handle_response(&self, response: Response) {
        println!("response: {:?}", response);
        let sequence = response.sequence.clone();
        if let Some(promise) = self.sequences.remove(&sequence) {
            let promise = promise.1;

            match response.status {
                Some(status) => {
                    if status != MessageState::OK.into() {
                        promise.lock().await.reject();
                        return;
                    }
                    match response.data.0 {
                        Some(data) => {
                            promise.lock().await.resolve(*data);
                        }
                        _ => {
                            promise.lock().await.resolve(Body::default());
                        }
                    }
                }
                None => todo!(),
            }
        }
    }

    pub async fn send(&self, data: &[u8]) -> Result<()> {
        let mut conn = self.conn.lock().await;

        match conn.send_message(&websocket::message::Message::binary(data)) {
            Ok(_) => {
                println!("send success")
            }
            Err(error) => {
                match reconnect(self.address.clone(), self.conn.clone()).await {
                    Ok(_) => {
                        conn = self.conn.lock().await;
                        conn.send_message(&websocket::message::Message::binary(data))?;
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

impl<R: Runtime> Drop for WClient<R> {
    fn drop(&mut self) {
        self.address = "".to_string();
        self.window
            .emit(&uniform_event_name("close"), "close")
            .unwrap();
        drop(&self.window);
        drop(&self.conn);
        drop(&self.sequences);
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
    conn: Arc<AsyncMutex<Client<TcpStream>>>,
    clients: Arc<RwLock<Vec<Arc<WClient<R>>>>>,
    conns: Arc<DashMap<String, Arc<AsyncMutex<Client<TcpStream>>>>>,
) {
    loop {
        let mut mconn = conn.lock().await;
        let close = || {
            // 从clients中删除 address 对应的client
            clients
                .write()
                .unwrap()
                .retain(|client| client.address != address);
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
                            }
                            MessageType::REQUEST => {
                                handle_message!(payload, index, Request, clients, address);
                            }
                            MessageType::RESPONSE => {
                                handle_message!(payload, index, Response, clients, address);
                            }
                            MessageType::OTHER => {
                                println!("recv: binary {:?}", payload);
                            }
                        };
                    }
                    websocket::message::OwnedMessage::Close(_) => {
                        println!("recv: close");
                        close();
                        mconn.shutdown().unwrap();
                    }
                    websocket::message::OwnedMessage::Ping(payload) => {
                        println!("recv: ping");
                        mconn
                            .send_message(&websocket::message::Message::pong(payload))
                            .unwrap();
                    }
                    websocket::message::OwnedMessage::Pong(_) => {
                        println!("recv: pong");
                        // conn.send_message(&websocket::message::Message::pong(payload)).unwrap();
                    }
                }
            }
            Err(_) => match reconnect(address.clone(), conn.clone()).await {
                Ok(_) => continue,
                Err(_) => {
                    close();
                    conns.remove(&address);
                    break;
                }
            },
        }
    }
}

async fn reconnect(address: String, conn: Arc<AsyncMutex<Client<TcpStream>>>) -> Result<(), ()> {
    // 重试10次
    let mut count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let res = ClientBuilder::new(address.as_str())
            .unwrap()
            .connect_insecure();
        match res {
            Ok(new_conn) => {
                let mut conn = conn.lock().await;
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
