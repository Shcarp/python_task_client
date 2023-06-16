use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::TcpStream;

use crate::{error::ConnectError, Conn, ConnBuilderConfig, Protocol};

#[derive(Debug, Clone)]
pub struct InnerTcpConn {
    pub ip: String,
    pub port: u16,
    pub protocol: Protocol,
    stream: Option<Arc<TcpStream>>,
}

unsafe impl Send for InnerTcpConn {}
unsafe impl Sync for InnerTcpConn {}

#[async_trait]
impl Conn for InnerTcpConn {
    fn new(config: ConnBuilderConfig) -> Self {
        Self {
            ip: config.host.clone(),
            port: config.port,
            protocol: Protocol::TCP,
            stream: None,
        }
    }

    async fn connect(&mut self) -> Result<bool, ConnectError> {
        return Ok(true);
    }
    async fn disconnect(&mut self) -> Result<bool, ConnectError> {
        return Ok(true);
    }
    async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError> {
        return Ok(true);
    }
    async fn receive(&mut self) -> Option<Vec<u8>> {
        return None;
    }
}
