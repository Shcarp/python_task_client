use anyhow::Result;
use conn::{ConnBuilder, ConnBuilderConfig, Connection, Protocol};
use dashmap::DashMap;
use proto::{message::Push, MessageType};
use protobuf::Message;
use std::sync::{Arc, RwLock};
use tauri::{Runtime, Window};

use crate::{
    client::utils::{CLIENT_IDENTIFICATION, CLIENT_IDENTIFICATION_CLOSE},
    wrap_event_err,
};
use log::error;

use super::{error::ConnError, w_client::WClient};

#[derive(Default)]
pub struct ClientManage<R: Runtime> {
    clients: Arc<RwLock<Vec<WClient<R>>>>,
    w_c: DashMap<String, WClient<R>>,
    conns: Arc<DashMap<String, Connection>>,
}

unsafe impl<R: Runtime> Send for ClientManage<R> {}
unsafe impl<R: Runtime> Sync for ClientManage<R> {}

impl<R: Runtime> ClientManage<R> {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
            w_c: DashMap::new(),
            conns: Arc::new(DashMap::new()),
        }
    }

    pub async fn add_client(&mut self, win: Window<R>, ip: String, port: u16) -> Result<String> {
        let address = format!("{}:{}", ip.clone(), port);
        println!("address: {}", address);
        match self.conns.get_mut(&address) {
            Some(conn) => {
                // 如果同一个窗口对应的address已经存在, 则不再添加
                if let Some(client) = self.w_c.get(win.label()) {
                    return Ok(client.client_id.clone());
                }
                let client = WClient::build(win, ip, port, conn.clone());

                let client_id = client.client_id.clone();
                self.clients
                    .write()
                    .map_err(|error| ConnError::LockError(error.to_string()))?
                    .push(client);
                Ok(client_id)
            }
            None => {
                let all_client = self.clients.clone();
                let connect_opt = ConnBuilderConfig {
                    host: ip.clone(),
                    port: port,
                    protocol: Protocol::WEBSOCKET,
                    error_callback: Box::new(move |ERR: String| {
                        for client in all_client.read().unwrap().iter() {
                            wrap_event_err!(
                                client.window,
                                CLIENT_IDENTIFICATION_CLOSE,
                                ERR.clone()
                            );
                        }
                    }),
                };
                let mut conn = ConnBuilder::new(connect_opt).build();
                conn.connect().await?;
                let recv_client = self.clients.clone();
                let conn_address = conn.get_address();
                let mut r_conn = conn.clone();
                // 开启任务读取数据
                tokio::spawn(async move {
                    loop {
                        let payload = r_conn.receive().await;
                        println!("payload: {:?}", payload);
                        match payload {
                            Ok(payload) => {
                                let mut index = 0;
                                // 读取第一个字节
                                let first_byte = payload[index];
                                // 转换为MessageType
                                let first_byte = first_byte as u8;
                                index += 1;
                                match MessageType::from_u8(first_byte) {
                                    MessageType::PUSH => {
                                        match Push::parse_from_bytes(&payload[index..]) {
                                            Ok(data) => {
                                                for client in recv_client.write().unwrap().iter_mut()
                                                {
                                                    if client.address == conn_address {
                                                        client.handle_push(data.clone())
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                error!("parse push error");
                                                // continue;
                                            }
                                        }
                                    }
                                    MessageType::REQUEST => {}
                                    MessageType::RESPONSE => {
                                        println!("response")
                                    }
                                    MessageType::OTHER => {}
                                };
                            }
                            Err(_) => {
                                error!("receive payload is None");
                                // continue;
                            }
                        }
                    }
                });

                let win_label = win.label().to_string();
                let client = WClient::build(win, ip.clone(), port, conn.clone());

                let client_id = client.client_id.clone();
                self.clients
                    .write()
                    .map_err(|error| ConnError::LockError(error.to_string()))?
                    .push(client.clone());
                self.w_c.insert(win_label, client.clone());
                self.conns.insert(address.to_string(), conn);
                Ok(client_id)
            }
        }
    }

    pub fn get_client(&self, client_id: String) -> Option<WClient<R>> {
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
                if let Some((_, mut conn)) = conns.remove(&address) {
                    conn.disconnect().await.unwrap();
                }
            }
        });

        wrap_event_err!(win, CLIENT_IDENTIFICATION_CLOSE, "close");
        Ok(())
    }
    pub async fn close_all(&mut self) -> Result<()> {
        let mut clients = self.clients.write().unwrap();
        self.conns.clear();
        clients.clear();
        Ok(())
    }
}
