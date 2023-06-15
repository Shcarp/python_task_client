mod inner_websocket;
use async_trait::async_trait;
pub use inner_websocket::InnerWebsocket;

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
    fn new(config: ConnBuilderConfig) -> Self;
    async fn connect(&mut self) -> bool;
    async fn disconnect(&mut self) -> bool;
    async fn send(&mut self, data: &[u8]) -> bool;
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

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn it_test_connect() {
        let connect_opt = ConnBuilderConfig {
            host: "127.0.0.1".to_string(),
            port: 9673,
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
}
