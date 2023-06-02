use std::sync::Mutex;
use anyhow::{Result};

use tauri::{
    plugin::{TauriPlugin},
    Manager, RunEvent, Runtime, State, Window
};

use crate::client::ClientManage;

#[derive(Debug, serde::Serialize)]
struct LResponse {
    code: u32,
    data: String,
}

impl Default for LResponse {
    fn default() -> Self {
        Self {
            code: 0,
            data: "".to_string(),
        }
    }
}

impl LResponse {
    fn new(code: u32, data: String) -> Self {
        Self {
            code,
            data,
        }
    }

    fn data(mut self, str: String) -> Self {
        self.data = str;
        self
    }

    fn code(mut self, code: u32) -> Self {
        self.code = code;
        self
    }
}

#[tauri::command]
fn connect<R: Runtime>(address: String, win: Window<R>, c_manage: State<'_, ClientState<R>>) -> Result<LResponse, LResponse> {
    println!("connect: {}", address);
    let res = c_manage.client_manage.lock().unwrap().add_client(win, &address);
    match res {
        Ok(id) => Ok(LResponse::default().data(id)),
        Err(err) => Err(LResponse::default().code(1).data(err.to_string()))
    }
}

#[tauri::command]
fn disconnect() {}

pub struct ClientState<R: Runtime> {
    client_manage: Mutex<ClientManage<R>>,
}

pub struct Builder {}

impl Default for Builder {
    fn default() -> Self {
        Self {}
    }
}

impl Builder {
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        tauri::plugin::Builder::new("connect")
            .invoke_handler(tauri::generate_handler![connect, disconnect])
            .setup(move |app_handle| {
                app_handle.manage(ClientState {
                    client_manage: Mutex::new(ClientManage::<R>::new())
                });
                Ok(())
            })
            .on_event(|app_handle, event| {
                if let RunEvent::Exit = event {
                    let manage = app_handle.state::<ClientState<R>>();
                    let manage = manage.client_manage.lock().unwrap().close_all();
                    if let Err(err) = manage {
                        println!("Failed to close client: {}", err);
                    }
                }
            })
            .build()
    }
}
