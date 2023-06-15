use async_trait::async_trait;

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    TCP,
    UDP,
    WEBSOCKET
}

pub struct WebsocketBuilder {
    pub host: String,
    pub port: u16,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::TCP
    }
}

/**
 * 保持连接
 * 断线重连
 * ping pong
 */
#[async_trait]
pub trait Conn {
    fn connect(target: WebsocketBuilder) -> Self;
    async fn disconnect(&self) -> bool;
    async fn send(&mut self, data: &[u8]) -> bool;
    async fn receive(&mut self) -> Option<Vec<u8>>;
}
