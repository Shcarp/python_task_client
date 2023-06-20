#[macro_export]
macro_rules! wrap_event_err {
    ($trigger:expr, $event:expr, $data:expr) => {
        if let Err(error) = $trigger.emit(&format!("{}::{}", CLIENT_IDENTIFICATION, $event), $data)
        {
            error!("emit error: {:?}", error);
        }
    };
}

#[macro_export]
macro_rules! handle_message {
    ($payload:expr, $index:expr, $message_type:ident, $clients:expr, $address:expr) => {
        match $message_type::parse_from_bytes(&$payload[$index..]) {
            Ok(data) => {
                let clients = $clients
                    .read()
                    .map_err(|error| ConnError::LockError(error.to_string()))
                    .unwrap();
                for client in recv_client.read().unwrap().iter_mut() {
                    if (client.address == recv_conn.get_address()) {
                    
                    }
                }
            }
            Err(error) => {
                error!("parse data error: {}", error);
            }
        }
    };
}
