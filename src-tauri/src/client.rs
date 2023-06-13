use anyhow::Result;
use async_recursion::async_recursion;
use dashmap::DashMap;
use log::{error, info};
use proto::{
    message::{Body, DataType, Push, Request, Response, Status as MessageState},
    MessageType,
};
use protobuf::{Message, SpecialFields};
use serde_json::Value;
use std::{
    any::Any,
    sync::atomic::{AtomicU8, Ordering},
    time::Duration,
};
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
    time::Instant,
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
const CLIENT_IDENTIFICATION_RESPONSE: &str = "CLIENT_IDENTIFICATION_RESPONSE";
const CLIENT_IDENTIFICATION_PUSH: &str = "CLIENT_IDENTIFICATION_PUSH";
const CLIENT_IDENTIFICATION_REQUEST: &str = "CLIENT_IDENTIFICATION_REQUEST";
const CLIENT_IDENTIFICATION_CLOSE: &str = "CLIENT_IDENTIFICATION_CLOSE";
const CLIENT_IDENTIFICATION_ERROR: &str = "CLIENT_IDENTIFICATION_ERROR";
const CLIENT_IDENTIFICATION_CONNECT_ERROR: &str = "CLIENT_IDENTIFICATION_CONNECT_ERROR";

const CONNECT_STATE_INIT: u8 = 0;
const CONNECT_STATE_CONNECTING: u8 = 1;
const CONNECT_STATE_CONNECTED: u8 = 2;
const CONNECT_STATE_CLOSED: u8 = 3;
const CONNECT_STATE_CLOSING: u8 = 4;
const CONNECT_STATE_RECONNECT: u8 = 5;

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

            client.conn_state.fetch_add(2, Ordering::Relaxed);
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

                            client
                                .conn_state
                                .swap(CONNECT_STATE_CONNECTING, Ordering::Relaxed);

                            let client_id = client.client_id.clone();

                            // 启动读取线程
                            let clients = self.clients.clone();
                            let conns = self.conns.clone();
                            let conn1 = conn.clone();
                            let address1 = address.to_string();
                            tokio::spawn(async move {
                                read_loop(address1, conn1, clients, conns).await
                            });

                            client
                                .conn_state
                                .swap(CONNECT_STATE_CONNECTED, Ordering::Relaxed);

                            self.clients
                                .write()
                                .map_err(|error| ConnError::LockError(error.to_string()))?
                                .push(client);

                            self.conns.insert(address.to_string(), conn.clone());
                            Ok(client_id)
                        }
                        Err(error) => {
                            wrap_event_err!(
                                win,
                                CLIENT_IDENTIFICATION_ERROR,
                                Some(&error.to_string())
                            );
                            Err(ConnError::ConnectError(error.to_string()).into())
                        }
                    }
                }
                Err(error) => {
                    wrap_event_err!(win, CLIENT_IDENTIFICATION_ERROR, Some(&error.to_string()));
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
        let mut address = String::new();
        self.clients.write().unwrap().retain(|client| {
            if client.client_id == client_id {
                std::mem::swap(&mut address, &mut client.address.clone());
                false
            } else {
                true
            }
        });

        let clients = self.clients.clone();
        let conns = self.conns.clone();
        tokio::spawn(async move {
            let mut num = 0;
            for client in clients
                .read()
                .map_err(|error| ConnError::LockError(error.to_string()))
                .unwrap()
                .iter()
            {
                if client.address == address {
                    num += 1;
                }
            }

            if num == 0 {
                if let Some((_, conn)) = conns.remove(&address) {
                    conn.writer.lock().await.shutdown().unwrap();
                    conn.reader.lock().await.shutdown().unwrap();
                }
            }
        });

        wrap_event_err!(win, CLIENT_IDENTIFICATION_CLOSE, "close");
        Ok(())
    }

    pub async fn close_all(&mut self) -> Result<()> {
        for client in self
            .clients
            .write()
            .map_err(|error| ConnError::LockError(error.to_string()))?
            .iter_mut()
        {
            client.close().await?;
        }
        self.clients = Arc::new(RwLock::new(vec![]));
        Ok(())
    }
}

pub struct WClient<R: Runtime> {
    pub conn_state: Arc<AtomicU8>,
    pub client_id: String,
    pub window: Window<R>,
    pub address: String,
    pub conn: Conn,
    pub sequences: DashMap<String, Arc<AsyncMutex<Sender<PromiseResult<Body>>>>>,
}

impl<R: Runtime> WClient<R> {
    pub fn new(win: Window<R>, address: String, conn: Conn) -> WClient<R> {
        let client = WClient {
            conn_state: Arc::new(AtomicU8::new(CONNECT_STATE_INIT)),
            client_id: Uuid::new_v4().to_string(),
            window: win,
            address: address,
            conn: conn,
            sequences: DashMap::new(),
        };
        client
    }

    pub async fn close(&self) -> Result<()> {
        self.conn_state
            .swap(CONNECT_STATE_CLOSING, Ordering::Relaxed);
        wrap_event_err!(self.window, CLIENT_IDENTIFICATION_CLOSE, "close");
        self.conn_state
            .swap(CONNECT_STATE_CLOSED, Ordering::Relaxed);
        Ok(())
    }

    pub async fn request(&self, url: String, data: Body) -> Result<Value, Value> {
        println!("request: {}", url);
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
                        wrap_event_err!(self.window, CLIENT_IDENTIFICATION_REQUEST, "success");
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
        wrap_event_err!(self.window, CLIENT_IDENTIFICATION_RESPONSE, "response");
        let sequence = response.sequence.clone();
        if let Some(sender) = self.sequences.remove(&sequence) {
            let sender = sender.1;
            match response.status {
                Some(status) => {
                    println!("response status: {:?}", status);
                    if status != MessageState::OK.into() {
                        match response.data.0 {
                            Some(data) => Promise::reject(sender, *data).await,
                            _ => Promise::reject(sender, Body::default()).await,
                        };
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
        wrap_event_err!(self.window, CLIENT_IDENTIFICATION_PUSH, push);
    }

    pub async fn handle_request(&self, request: Request) {
        println!("request: {:?}", request);
    }

    #[async_recursion]
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        if self.conn_state.load(Ordering::Relaxed) != CONNECT_STATE_CONNECTED {
            return Err(ConnError::SendError("not connected".to_string()).into());
        }

        let mut writer = self.conn.writer.lock().await;
        match writer.send_message(&websocket::message::Message::binary(data)) {
            Ok(_) => {
                drop(writer);
                Ok(())
            }
            Err(error) => {
                drop(writer);
                if self.conn_state.load(Ordering::Relaxed) == CONNECT_STATE_RECONNECT {
                    // 等到重连成功后再发送
                    self.await_reconnect().await?;
                    if self.conn_state.load(Ordering::Relaxed) == CONNECT_STATE_CONNECTED {
                        return self.send(data).await;
                    } else {
                        let err = ConnError::SendError(error.to_string());
                        wrap_event_err!(
                            self.window,
                            CLIENT_IDENTIFICATION_CONNECT_ERROR,
                            err.to_string()
                        );
                        return Err(err.into());
                    }
                }

                self.conn_state
                    .swap(CONNECT_STATE_RECONNECT, Ordering::Relaxed);
                match reconnect(self.address.clone(), self.conn.clone()).await {
                    Ok(_) => {
                        return self.send(data).await;
                    }
                    Err(_) => {
                        let err = ConnError::SendError(error.to_string());
                        wrap_event_err!(
                            self.window,
                            CLIENT_IDENTIFICATION_CONNECT_ERROR,
                            err.to_string()
                        );
                        return Err(err.into());
                    }
                }
            }
        }
    }

    async fn await_reconnect(&self) -> Result<(), ConnError> {
        let mut reconnect_count = 0;
        while self.conn_state.load(Ordering::Relaxed) == CONNECT_STATE_RECONNECT {
            let deadline = Instant::now() + Duration::from_secs(1);
            tokio::time::sleep_until(deadline).await;
            reconnect_count += 1;
            if reconnect_count > 8 {
                return Err(ConnError::SendError("reconnect timeout".to_string()).into());
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

        let set_state = |val: u8| {
            let mut clients = clients.write().unwrap();
            for client in clients.iter_mut() {
                if client.address == address {
                    client.conn_state.swap(val, Ordering::Relaxed);
                }
            }
        };

        let close = |state: &str| {
            set_state(CONNECT_STATE_CLOSING);
            // 从clients中删除 address 对应的client
            clients.write().unwrap().retain(|client| {
                if client.address == address {
                    wrap_event_err!(
                        client.window,
                        state,
                        "connect error"
                    );
                }
                client.address != address
            });
            set_state(CONNECT_STATE_CLOSED);
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
                        close(CLIENT_IDENTIFICATION_CLOSE);
                        reader.shutdown().unwrap();
                        return;
                    }
                    websocket::message::OwnedMessage::Ping(payload) => {
                        let mut writer = conn.writer.lock().await;
                        writer
                            .send_message(&websocket::message::Message::pong(payload))
                            .unwrap();
                    }
                    websocket::message::OwnedMessage::Pong(_) => {
                        info!("recv: pong");
                    }
                }
            }

            Err(_) => {
                drop(reader);
                set_state(CONNECT_STATE_RECONNECT);
                match reconnect(address.clone(), conn.clone()).await {
                    Ok(_) => {
                        set_state(CONNECT_STATE_CONNECTED);
                    }
                    Err(_) => {
                        set_state(CONNECT_STATE_CLOSING);
                        close(CLIENT_IDENTIFICATION_CONNECT_ERROR);
                        set_state(CONNECT_STATE_CLOSED);
                        conns.remove(&address);
                        break;
                    }
                }
            }
        }
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
                if count > 10 {
                    return Err(());
                }
            }
        }
    }
}
