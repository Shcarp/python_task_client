use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use conn::Connection;
use dashmap::DashMap;
use promise::{self, Promise, PromiseResult};
use proto::{
    message::{Body, Push, Request, Response, Status as MessageState},
    MessageBody, MessageType,
};
use protobuf::Message;
use serde_json::Value;
use tauri::{Runtime, Window};
use uuid::Uuid;

use crate::client::utils::{
    CLIENT_IDENTIFICATION, CLIENT_IDENTIFICATION_REQUEST, CLIENT_IDENTIFICATION_RESPONSE, CLIENT_IDENTIFICATION_PUSH, CLIENT_IDENTIFICATION_ERROR,
};
use log::{error, info};

use crate::wrap_event_err;

#[derive(Clone)]
pub enum RecvData {
    Error(String),
    Push(Push),
    Request(Request),
    Response(Response),
}

unsafe impl Send for RecvData {}
unsafe impl Sync for RecvData {}
    
pub struct WClient<R: Runtime> {
    pub ip: String,
    pub port: u16,
    pub address: String,
    pub client_id: String,
    pub window: Window<R>,
    pub conn: Connection,
    pub sequences: Arc<DashMap<String, Promise<Body>>>,
}

unsafe impl<R: Runtime> Send for WClient<R> {}
unsafe impl<R: Runtime> Sync for WClient<R> {}

impl<R: Runtime> Clone for WClient<R> {
    fn clone(&self) -> Self {
        Self {
            client_id: self.client_id.clone(),
            window: self.window.clone(),
            conn: self.conn.clone(),
            address: self.address.clone(),
            sequences: self.sequences.clone(),
            ip: self.ip.clone(),
            port: self.port,
        }
    }
}

impl<R: Runtime> WClient<R> {
    pub fn build(window: Window<R>, ip: String, port: u16, conn: Connection) -> Self {
        Self {
            client_id: Uuid::new_v4().to_string(),
            window,
            ip: ip.clone(),
            port: port,
            address: format!("{}:{}", ip, port),
            conn,
            sequences: Arc::new(DashMap::new()),
        }
    }

    pub async fn request(&mut self, url: String, data: Body) -> Result<Value, Value> {
        let mut promise = Promise::<Body>::new();
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
                match self.conn.send(&data).await {
                    Ok(_) => {
                        wrap_event_err!(self.window, CLIENT_IDENTIFICATION_REQUEST, "success");
                    }
                    Err(error) => {
                        error!("request error: {:?}", error);
                        self.sequences.remove(&sequence);
                        promise.reject(Body::from_serialize(Value::String(format!(
                            "request error: {:?}",
                            error
                        )))).await.unwrap();
                    }
                }
                self.sequences.insert(sequence.clone(), promise.clone());

                let mut t_promise = promise.clone();
                let handle = tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    t_promise.reject(Body::from_serialize(Value::String("timeout".to_string()))).await.unwrap();
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

    pub async fn handle_response(&mut self, response: Response) {
        wrap_event_err!(self.window, CLIENT_IDENTIFICATION_RESPONSE, "response");
        let sequence = response.sequence.clone();
        if let Some(promise) = self.sequences.remove(&sequence) {
            let mut promise = promise.1;
            match response.status {
                Some(status) => {
                    if status != MessageState::OK.into() {
                        match response.data.0 {
                            Some(data) => promise.reject(*data).await.unwrap(),
                            _ => promise.reject(Body::default()).await.unwrap(),
                        };
                        return;
                    }
                    match response.data.0 {
                        Some(data) => {
                            promise.resolve(*data).await.unwrap();
                        }
                        _ => {
                            promise.resolve(Body::default()).await.unwrap();
                        }
                    }
                }
                None => {
                    info!("response status is none")
                }
            }
        }
    }

    pub fn handle_push(&mut self, data: Push) {
        wrap_event_err!(self.window, CLIENT_IDENTIFICATION_PUSH, data);
    }

    pub fn handle_error(&mut self, data: String) {
        wrap_event_err!(self.window, CLIENT_IDENTIFICATION_ERROR, data)
    }

    pub fn handle_message(&mut self, data: RecvData) {
        let mut this = self.clone();
        tokio::spawn(async move {
            match data {
                RecvData::Push(data) => {
                    this.handle_push(data);
                },
                RecvData::Response(data) => {
                    this.handle_response(data).await;
                },
                RecvData::Error(data) => {
                    this.handle_error(data);
                },
                _ => {
                    info!("handle message");
                }
            }
        });
    }
}
