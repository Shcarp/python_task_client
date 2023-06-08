use anyhow::Result;
use dashmap::DashMap;
use log::{error, info};
use proto::{
    message::{Body, DataType, Push, Request, Response, Status as MessageState},
    MessageType,
};
use protobuf::{Message, SpecialFields};
use serde_json::Value;
use std::any::Any;
use std::{
    fmt::Debug,
    mem,
    net::TcpStream,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{Runtime, Window};
use thiserror::Error;
use tokio::{
    sync::{mpsc::Sender, Mutex as AsyncMutex},
};
use uuid::Uuid;
use websocket::{
    sync::{Reader, Writer},
    ClientBuilder,
};

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

macro_rules! wrap_event_err {
    ($trigger:expr, $event:expr, $data:expr) => {
        if let Err(error) = $trigger.emit(&uniform_event_name($event), $data) {
            error!("emit error: {:?}", error);
        }
    };
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
    w_c: DashMap<String, Arc<WClient<R>>>,
    conns: Arc<DashMap<String, Conn>>,
}

impl<R: Runtime> ClientManage<R> {
    pub fn new() -> ClientManage<R> {
        ClientManage {
            clients: Arc::new(RwLock::new(vec![])),
            conns: Arc::new(DashMap::new()),
            w_c: DashMap::new(),
        }
    }

    pub fn add_client(&mut self, win: Window<R>, address: &str) -> Result<String> {
        // 判断address是否已经存在, 如果已存在则用存在的conn
        if let Some(conn) = self.conns.get(address) {
            // 如果同一个窗口对应的address已经存在, 则不再添加
            if let Some(client) = self.w_c.get(win.label()) {
                return Ok(client.client_id.clone());
            }
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
                            let client =
                                Arc::new(WClient::new(win, address.to_string(), conn.clone()));
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
                                async move { read_loop(address, conn, clients, conns).await },
                            );
                            Ok(client_id)
                        }
                        Err(error) => {
                            wrap_event_err!(win, "error", Some(&error.to_string()));
                            Err(ConnError::ConnectError(error.to_string()).into())
                        }
                    }
                }
                Err(error) => {
                    wrap_event_err!(win, "error", Some(&error.to_string()));
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
        wrap_event_err!(win, "close", "close");
        Ok(())
    }

    pub async fn close_all(&mut self) -> Result<()> {
        for client in self
            .clients
            .write()
            .map_err(|error| ConnError::LockError(error.to_string()))?
            .iter()
        {
            wrap_event_err!(client.window, "close", "close");
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
    pub sequences: DashMap<String, Arc<AsyncMutex<Sender<PromiseResult<Body>>>>>,
}

impl<R: Runtime> WClient<R> {
    pub fn new(win: Window<R>, address: String, conn: Conn) -> WClient<R> {
        let client = WClient {
            client_id: Uuid::new_v4().to_string(),
            window: win,
            address: address,
            conn: conn,
            sequences: DashMap::new(),
        };
        client
    }

    pub async fn request(&self, url: String, data: Body) -> Result<Value, Value> {
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
                        wrap_event_err!(self.window, "send", "success");
                    }
                    Err(error) => {
                        error!("request error: {:?}", error);
                        self.sequences.remove(&sequence);
                        sender
                            .lock()
                            .await
                            .send(PromiseResult::Rejected(Body::from_serialize(
                                Value::String(format!("request error: {:?}", error)),
                            )))
                            .await
                            .unwrap();
                    }
                }
                self.sequences.insert(sequence.clone(), sender.clone());

                let t_sender = sender.clone();
                let handle = tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    t_sender
                        .lock()
                        .await
                        .send(PromiseResult::Rejected(Body::from_serialize(
                            Value::String("timeout".to_string()),
                        )))
                        .await
                        .unwrap();
                });

                let res = match promise.await {
                    PromiseResult::Resolved(value) => Ok(value.json_value()),
                    PromiseResult::Rejected(value) => Err(value.json_value()),
                };
                // 清除定时器
                handle.abort();
                res
            }
            Err(_) => Err(Value::String("data error".to_string())),
        }
    }

    pub async fn handle_response(&self, response: Response) {
        let sequence = response.sequence.clone();
        if let Some(sender) = self.sequences.remove(&sequence) {
            let sender = sender.1;
            match response.status {
                Some(status) => {
                    if status != MessageState::OK.into() {
                        Promise::reject(
                            sender,
                            Body::from_serialize(Value::String("request error".to_string())),
                        )
                        .await;
                        return;
                    }
                    match response.data.0 {
                        Some(data) => {
                            Promise::resolve(sender.clone(), *data).await;
                        }
                        _ => {
                            Promise::resolve(sender, Body::default()).await;
                        }
                    }
                }
                None => {
                    info!("response status is none")
                }
            }
        }
    }

    pub async fn handle_push(&self, push: Push) {
        wrap_event_err!(self.window, "push", push);
    }

    pub async fn handle_request(&self, request: Request) {
        println!("request: {:?}", request);
    }

    pub async fn send(&self, data: &[u8]) -> Result<()> {
        let mut writer = self.conn.writer.lock().await;
        match writer.send_message(&websocket::message::Message::binary(data)) {
            Ok(_) => {}
            Err(error) => {
                drop(writer);
                match reconnect(self.address.clone(), self.conn.clone()).await {
                    Ok(_) => {
                        let mut writer = self.conn.writer.lock().await;
                        writer.send_message(&websocket::message::Message::binary(data))?;
                    }
                    Err(_) => {
                        let err = ConnError::SendError(error.to_string());
                        wrap_event_err!(self.window, "close", err.to_string());
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
    conns: Arc<DashMap<String, Conn>>,
) {
    loop {
        let mut reader = conn.reader.lock().await;
        let close = || {
            // 从clients中删除 address 对应的client
            clients.write().unwrap().retain(|client| {
                if client.address == address {
                    wrap_event_err!(client.window, "CONNECT_CLOSE", {});
                }
                client.address != address
            });
        };

        match reader.recv_message() {
            Ok(response) => {
                match response {
                    websocket::message::OwnedMessage::Text(text) => {
                        info!("recv: {}", text);
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
                                info!("Not found");
                            }
                        };
                    }
                    websocket::message::OwnedMessage::Close(_) => {
                        info!("CONN CLOSE");
                        close();
                        reader.shutdown().unwrap();
                    }
                    websocket::message::OwnedMessage::Ping(payload) => {
                        let mut writer = conn.writer.lock().await;
                        writer
                            .send_message(&websocket::message::Message::pong(payload))
                            .unwrap();
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
                    }
                    Err(_) => {
                        return Err(());
                    }
                }
            }
            Err(error) => {
                error!("reconnect error: {}", error);
                count += 1;
                if count > 100 {
                    return Err(());
                }
            }
        }
    }
}
