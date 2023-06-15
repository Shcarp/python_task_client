mod inner_websocket;
use async_trait::async_trait;
use inner_websocket::InnerWebsocket;

#[derive(Debug, Clone)]
pub enum Protocol {
    TCP,
    UDP,
    WEBSOCKET,
}

pub struct ConnBuilderConfig {
    pub host: String,
    pub port: u16,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::TCP
    }
}

#[async_trait]
pub trait Conn {
    async fn connect(&mut self) -> bool;
    async fn disconnect(&mut self) -> bool;
    async fn send(&mut self, data: &[u8]) -> bool;
    async fn receive(&mut self) -> Option<Vec<u8>>;
}

pub struct Connect(Box<dyn Conn + Send + Sync>);

unsafe impl Sync for Connect {}
unsafe impl Send for Connect {}

#[async_trait]
impl Conn for Connect {
    async fn connect(&mut self) -> bool {
        return self.0.connect().await;
    }
    async fn disconnect(&mut self) -> bool {
        return self.0.disconnect().await;
    }
    async fn send(&mut self, data: &[u8]) -> bool {
        return self.0.send(data).await;
    }
    async fn receive(&mut self) -> Option<Vec<u8>> {
        return self.0.receive().await;
    }
}

pub fn make_connect(protocol: Protocol, config: ConnBuilderConfig) -> Connect {
    let connection: Box<dyn Conn + Send + Sync> = match protocol {
        Protocol::TCP => Box::new(InnerWebsocket::new(config)),
        Protocol::UDP => Box::new(InnerWebsocket::new(config)),
        Protocol::WEBSOCKET => Box::new(InnerWebsocket::new(config)),
    };
    return Connect(connection);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn it_test_connect() {
        let connect_opt = ConnBuilderConfig {
            host: "127.0.0.1".to_string(),
            port: 9673,
        };

        let mut conn = make_connect(Protocol::WEBSOCKET, connect_opt);
        conn.connect().await;

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
}
