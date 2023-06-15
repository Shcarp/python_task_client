use conn::{Conn, ConnBuilderConfig, make_connect};

#[tokio::main]
async fn main() {
    let connect_opt = ConnBuilderConfig {
        host: "127.0.0.1".to_string(),
        port: 9673,
    };

    let mut conn = make_connect(conn::Protocol::WEBSOCKET, connect_opt);
    conn.connect().await;

    loop {
        println!("loop");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let message = conn.receive().await;
        match message {
            Some(message) => {
                println!("message: {:?}", message)
            },
            None => {
                println!("message: None")
            }
        }
    }
}