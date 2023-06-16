mod error;
mod inner_tcp;
mod inner_udp;
mod inner_websocket;
use std::fmt::Debug;

use async_trait::async_trait;
use error::ConnectError;
pub use inner_websocket::InnerWebsocket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    TCP,
    UDP,
    WEBSOCKET,
}

pub struct ConnBuilderConfig {
    pub host: String,
    pub port: u16,
    pub error_callback: Box<dyn FnMut(String) + Send + Sync>,
}

impl Debug for ConnBuilderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnBuilderConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .finish()
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::TCP
    }
}

#[async_trait]
pub trait Conn {
    fn new(config: ConnBuilderConfig) -> Self;
    async fn connect(&mut self) -> Result<bool, ConnectError>;
    async fn disconnect(&mut self) -> Result<bool, ConnectError>;
    async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError>;
    async fn receive(&mut self) -> Option<Vec<u8>>;
}
#[derive(Clone)]
pub struct Connect<T: Conn + Send + Sync + Clone>(T);

unsafe impl<T: Conn + Send + Sync + Clone> Sync for Connect<T> {}
unsafe impl<T: Conn + Send + Sync + Clone> Send for Connect<T> {}

#[async_trait]
impl<T: Conn + Send + Sync + Clone> Conn for Connect<T> {
    fn new(config: ConnBuilderConfig) -> Self {
        Connect(T::new(config))
    }
    async fn connect(&mut self) -> Result<bool, ConnectError> {
        return self.0.connect().await;
    }
    async fn disconnect(&mut self) -> Result<bool, ConnectError> {
        return self.0.disconnect().await;
    }
    async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError> {
        return self.0.send(data).await;
    }
    async fn receive(&mut self) -> Option<Vec<u8>> {
        return self.0.receive().await;
    }
}

#[cfg(test)]
mod tests {

    use crate::{inner_tcp::InnerTcpConn, inner_udp::InnerUdpConn};

    use super::*;

    #[tokio::test]
    async fn it_test_connect() {
        let connect_opt = ConnBuilderConfig {
            host: "127.0.0.1".to_string(),
            port: 9673,
            error_callback: Box::new(|ERR: String| {
                println!("ERR: {}", ERR);
            }),
        };

        let mut conn = Connect::<InnerWebsocket>::new(connect_opt);
        conn.connect().await;

        let mut new_conn = conn.clone();

        tokio::spawn(async move {
            new_conn.send(&[49]).await;
        });

        loop {
            println!("loop");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let message = conn.receive().await;
            match message {
                Some(message) => {
                    println!("message: {:?}", message)
                }
                None => {
                    println!("message: None")
                }
            }
        }
    }

    #[tokio::test]
    async fn it_test_kind_of_connect() {
        let connect_opt = ConnBuilderConfig {
            host: "127.0.0.1".to_string(),
            port: 9673,
            error_callback: Box::new(|ERR: String| {
                println!("error");
            }) 
        };

        // let web_conn = Connect::<InnerWebsocket>::new(connect_opt.clone());
        // let tcp_conn = Connect::<InnerTcpConn>::new(connect_opt.clone());
        // let udp_conn = Connect::<InnerUdpConn>::new(connect_opt.clone());

        // assert_eq!(web_conn.0.protocol, Protocol::WEBSOCKET);
        // assert_eq!(tcp_conn.0.protocol, Protocol::TCP);
        // assert_eq!(udp_conn.0.protocol, Protocol::UDP);

        // assert_eq!(web_conn.0.port, 9673);
        // assert_eq!(tcp_conn.0.port, 9673);
        // assert_eq!(udp_conn.0.port, 9673);

        // assert_eq!(web_conn.0.ip, "127.0.0.1");
        // assert_eq!(tcp_conn.0.ip, "127.0.0.1");
        // assert_eq!(udp_conn.0.ip, "127.0.0.1");
    }
}
