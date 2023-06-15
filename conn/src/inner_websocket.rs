use async_trait::async_trait;
use std::{
    fmt::Debug,
    net::TcpStream,
    sync::{atomic::AtomicU64, Arc},
    sync::{
        atomic::{AtomicU8, Ordering},
        RwLock,
    },
};
use tokio::sync::{mpsc::{channel, Sender, Receiver}, Mutex};
use websocket::{
    sync::{Reader, Writer},
    ClientBuilder, OwnedMessage,
};

use crate::base::{Conn, Protocol, WebsocketBuilder};

const HEARTBEAT_INTERVAL: u64 = 55 * 1000;

const PING: &[u8] = b"ping";

const CONNECT_STATE_INIT: u8 = 0;
const CONNECT_STATE_CONNECTING: u8 = 1;
const CONNECT_STATE_CONNECTED: u8 = 2;
const CONNECT_STATE_CLOSED: u8 = 3;
const CONNECT_STATE_CLOSING: u8 = 4;
const CONNECT_STATE_RECONNECT: u8 = 5;

pub struct Websocket {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
    state: Arc<AtomicU8>,
    reader: Arc<Mutex<Reader<TcpStream>>>,
    writer: Arc<Mutex<Writer<TcpStream>>>,
    last_heartbeat: Arc<AtomicU64>,
    conn_task: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
    recv_channel: Option<Receiver<Vec<u8>>>,
}

impl Clone for Websocket {
    fn clone(&self) -> Self {
        Self {
            host: self.host.clone(),
            port: self.port.clone(),
            protocol: self.protocol.clone(),
            state: self.state.clone(),
            reader: self.reader.clone(),
            writer: self.writer.clone(),
            last_heartbeat: self.last_heartbeat.clone(),
            conn_task: self.conn_task.clone(),
            recv_channel: None,
        }
    }
}

impl Debug for Websocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Websocket")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("protocol", &self.protocol)
            .finish()
    }
}

impl Drop for Websocket {
    fn drop(&mut self) {
        let mut tasks = self.conn_task.write().unwrap();
        tasks.iter().for_each(|task| {
            task.abort();
        });
        tasks.clear();
        drop(tasks);
    }
}

impl Websocket {
    fn get_state(&self) -> u8 {
        self.state.load(Ordering::Relaxed)
    }

    // 开始发送心跳信息
    fn start_heartbeat(&self) {
        let heartbeat = self.writer.clone();

        let mut start_time = chrono::Utc::now().timestamp_millis() as u64;

        let heartbeat_task = tokio::spawn(async move {
            loop {
                let mut write = heartbeat.lock().await;
                let now = chrono::Utc::now().timestamp_millis() as u64;
                if now - start_time >= HEARTBEAT_INTERVAL {
                    write.send_message(&websocket::Message::ping(PING)).unwrap();
                    start_time = chrono::Utc::now().timestamp_millis() as u64;
                }
                drop(write);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
        self.conn_task.write().unwrap().push(heartbeat_task);
    }

    fn start_reader(&mut self, sender: Sender<Vec<u8>>) {
        let shared_self = Arc::new(Mutex::new(self.clone()));

        let recv_reader = self.reader.clone();
        let recv_writer = self.writer.clone();
        let recv_time = self.last_heartbeat.clone();

        let recv_task = tokio::spawn(async move {
            loop {
                if (shared_self.lock().await.get_state() == CONNECT_STATE_CLOSED)
                    || (shared_self.lock().await.get_state() == CONNECT_STATE_CLOSING)
                {
                    println!("连接已断开");
                    break;
                }
                let mut read = recv_reader.lock().await;
                let now = chrono::Utc::now().timestamp_millis() as u64;
                recv_time.store(now, std::sync::atomic::Ordering::Relaxed);
                match read.recv_message() {
                    Ok(res) => match res {
                        OwnedMessage::Binary(payload) => {
                            println!("收到二进制信息 {:?}", payload);
                            println!("{}", sender.capacity());
                            sender.send(payload).await.expect("接收信息失败")
                        }
                        OwnedMessage::Ping(payload) => {
                            println!("收到心跳信息 ping");
                            recv_writer
                                .lock()
                                .await
                                .send_message(&websocket::Message::pong(payload))
                                .unwrap();
                        }
                        OwnedMessage::Pong(_) => {
                            println!("收到心跳信息 pong");
                        }
                        _ => {
                            println!("收到信息");
                        }
                    },
                    Err(_) => {
                        println!("接收信息失败");
                        drop(read);
                        let mut s_shelf = shared_self.lock().await;
                        match s_shelf.reconnect().await {
                            Ok(_) => {
                                println!("重连成功");
                                continue;
                            }
                            Err(_) => {
                                println!("重连失败");
                                s_shelf.state.store(CONNECT_STATE_CLOSED, Ordering::Relaxed);
                            }
                        };
                        drop(s_shelf)
                    }
                }
            }
        });
        self.conn_task.write().unwrap().push(recv_task);
    }

    async fn reconnect(&mut self) -> Result<(), ()> {
        if (self.get_state() == CONNECT_STATE_CLOSED)
            || (self.get_state() == CONNECT_STATE_CLOSING)
            || (self.get_state() == CONNECT_STATE_RECONNECT)
        {
            return Ok(());
        }
        let mut count = 0;
        self.state.store(CONNECT_STATE_RECONNECT, Ordering::Relaxed);
        loop {
            self.conn_task.write().unwrap().clear();
            std::thread::sleep(std::time::Duration::from_secs(10));
            let (reader, writer) =
                match ClientBuilder::new(&format!("ws://{}:{}", self.host, self.port))
                    .unwrap()
                    .connect_insecure()
                {
                    Ok(conn) => conn.split().unwrap(),
                    Err(error) => {
                        println!("连接失败: {}", error);
                        count += 1;
                        if count >= 10 {
                            self.state.store(CONNECT_STATE_CLOSED, Ordering::Relaxed);
                            return Err(());
                        }
                        continue;
                    }
                };

            let mut reader_guard = self.reader.lock().await;
            let mut writer_guard = self.writer.lock().await;

            let old_reader = std::mem::replace(&mut *reader_guard, reader);
            let old_writer = std::mem::replace(&mut *writer_guard, writer);

            drop(old_reader);
            drop(old_writer);

            self.state.store(CONNECT_STATE_CONNECTED, Ordering::Relaxed);
            return Ok(());
        }
    }
}

#[async_trait]
impl Conn for Websocket {
    fn connect(target: WebsocketBuilder) -> Self {
        let state = Arc::new(AtomicU8::new(CONNECT_STATE_INIT));
        state.store(CONNECT_STATE_CONNECTING, Ordering::Relaxed);
        let origin_conn = ClientBuilder::new(&format!("ws://{}:{}", target.host, target.port))
            .or_else(|err| {
                state.store(CONNECT_STATE_CLOSED, Ordering::Relaxed);
                Err(err)
            })
            .unwrap()
            .connect_insecure()
            .or_else(|err| {
                state.store(CONNECT_STATE_CLOSED, Ordering::Relaxed);
                Err(err)
            })
            .unwrap();
        let (reader, writer) = origin_conn.split().unwrap();
        let reader = Arc::new(Mutex::new(reader));
        let writer = Arc::new(Mutex::new(writer));

        let (sender, receiver) = channel::<Vec<u8>>(10);

        let mut ws = Websocket {
            host: target.host,
            port: target.port,
            state: state,
            protocol: Protocol::WEBSOCKET,
            reader,
            writer,
            last_heartbeat: Arc::new(AtomicU64::new(chrono::Utc::now().timestamp_millis() as u64)),
            conn_task: Arc::new(RwLock::new(vec![])),
            recv_channel: Option::Some(receiver),
        };
        ws.start_heartbeat();
        ws.start_reader(sender);
        ws.state.swap(CONNECT_STATE_CONNECTED, Ordering::Relaxed);
        ws
    }

    async fn disconnect(&self) -> bool {
        self.state.store(CONNECT_STATE_CLOSING, Ordering::Relaxed);
        let mut send = self.writer.lock().await;
        let mut tasks = self.conn_task.write().unwrap();
        tasks.iter().for_each(|task| {
            task.abort();
        });
        tasks.clear();
        drop(tasks);
        match send.send_message(&websocket::Message::close()) {
            Ok(_) => {
                self.state.store(CONNECT_STATE_CLOSED, Ordering::Relaxed);
                true
            }
            Err(_) => {
                drop(send);
                false
            }
        }
    }

    async fn send(&mut self, data: &[u8]) -> bool {
        let mut send = self.writer.lock().await;
        match send.send_message(&websocket::Message::binary(data)) {
            Ok(_) => true,
            Err(_) => {
                drop(send);
                if self.get_state() == CONNECT_STATE_CONNECTED {
                    // 重连
                    match self.reconnect().await {
                        Ok(_) => {
                            if self.send(data).await {
                                return true;
                            }
                            return false;
                        }
                        Err(_) => {
                            return false;
                        }
                    }
                }
                false
            }
        }
    }

    async fn receive(&mut self) -> Option<Vec<u8>> {
        // 从通道中获取数据
        if let Some(ref mut receiver) = self.recv_channel {
            return receiver.recv().await;
        }
        None
    }
}
