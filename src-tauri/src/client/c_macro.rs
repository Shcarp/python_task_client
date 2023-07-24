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
               for client in
                    $clients.write().unwrap().iter_mut()
               {
                   if &client.address == $address {
                       client.handle_message(RecvData::$message_type(
                           data.clone(),
                       ));
                   }
               }
           }
           Err(_) => {
               error!("parse push error");
           }
       }
    };
}
