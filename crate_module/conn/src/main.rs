use conn::{Conn, ConnBuilderConfig, InnerWebsocket, Connect};

#[tokio::main]
async fn main() {
    let connect_opt = ConnBuilderConfig {
        host: "127.0.0.1".to_string(),
        port: 9673,
        error_callback: Box::new(|ERR: String| {
            println!("ERR: {}", ERR);
        }),
    };

    let mut conn = Connect::<InnerWebsocket>::new(connect_opt);
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