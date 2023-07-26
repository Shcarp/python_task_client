use anyhow::Result;
use dashmap::DashMap;
use proto::{
    message::{Push, Request, Response},
    MessageType,
};
use protobuf::Message;
use rs_connections::{ConnBuilder, ConnBuilderConfig, Protocol, Conn, ConnectionInterface, ConnectionBaseInterface, ERROR_EVENT, Emitter, EventHandler, ConnectError};
use std::sync::{Arc, RwLock};
use tauri::{Runtime, Window};

use crate::{
    client::{
        utils::{CLIENT_IDENTIFICATION, CLIENT_IDENTIFICATION_CLOSE},
        w_client::RecvData,
    },
    handle_message, wrap_event_err,
};
use log::error;

use super::{error::ConnError, w_client::WClient};

#[derive(Default)]
pub struct ClientManage<R: Runtime> {
    clients: Arc<RwLock<Vec<WClient<R>>>>,
    label_client: DashMap<String, WClient<R>>,
    conns: Arc<DashMap<String, Conn>>,
}

unsafe impl<R: Runtime> Send for ClientManage<R> {}
unsafe impl<R: Runtime> Sync for ClientManage<R> {}

impl<R: Runtime> ClientManage<R> {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
            label_client: DashMap::new(),
            conns: Arc::new(DashMap::new()),
        }
    }

    pub async fn add_client(&mut self, win: Window<R>, ip: String, port: u16) -> Result<String> {
        let address = format!("{}:{}", ip.clone(), port);
        match self.conns.get_mut(&address) {
            Some(conn) => {
                // 如果同一个窗口对应的address已经存在, 则不再添加
                if let Some(client) = self.label_client.get(win.label()) {
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
                let addr = address.clone();
                let connect_opt = ConnBuilderConfig {
                    host: ip.clone(),
                    port: port,
                    heartbeat_time: None,
                    protocol: Protocol::WEBSOCKET,
                };
                let mut conn = ConnBuilder::new(connect_opt).build();

                let handle_error = EventHandler::new(Box::new(move |data: ConnectError| {
                    for client in all_client.write().unwrap().iter_mut() {
                        if client.address == addr {
                            client.handle_message(RecvData::Error(data.to_string()));
                        }
                    }
                }));

                conn.on(ERROR_EVENT, Arc::new(handle_error.clone()));

                conn.connect().await?;
                let recv_client = self.clients.clone();
                let mut r_conn = conn.clone();
                // 开启任务读取数据
                tokio::spawn(async move {
                    loop {
                        let payload = r_conn.receive().await;
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
                                        handle_message!(
                                            payload,
                                            index,
                                            Push,
                                            recv_client,
                                            &r_conn.get_address()
                                        )
                                    }
                                    MessageType::REQUEST => {
                                        handle_message!(
                                            payload,
                                            index,
                                            Request,
                                            recv_client,
                                            &r_conn.get_address()
                                        )
                                    }
                                    MessageType::RESPONSE => {
                                        handle_message!(
                                            payload,
                                            index,
                                            Response,
                                            recv_client,
                                            &r_conn.get_address()
                                        )
                                    }
                                    MessageType::OTHER => {}
                                };
                            }
                            Err(_) => {
                                error!("receive payload is None");
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
                self.label_client.insert(win_label, client.clone());
                self.conns.insert(address.to_owned(), conn);
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
    pub fn close_all(&mut self) -> Result<()> {
        let mut clients = self.clients.write().unwrap();
        self.conns.clear();
        clients.clear();
        Ok(())
    }
}