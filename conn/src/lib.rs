pub mod base;
pub mod inner_websocket;

#[cfg(test)]
mod tests {
    use crate::base::Conn;

    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn it_test_connect() {
        let connect_opt = base::WebsocketBuilder {
            host: "127.0.0.1".to_string(),
            port: 9673,
        };
        let mut conn = inner_websocket::Websocket::connect(connect_opt);

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
}
