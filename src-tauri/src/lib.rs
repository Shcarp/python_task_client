pub mod command;

use tauri::Wry;

use command::generate_unique_message_id;
use tauri_plugin_log::{LogTarget};

pub struct NApp {
    builder: tauri::Builder<Wry>,
}

impl NApp {
    pub fn new() -> NApp {
        let builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![generate_unique_message_id])
        .plugin(tauri_plugin_log::Builder::default().targets([
            LogTarget::LogDir,
            LogTarget::Stdout,
            LogTarget::Webview,
        ]).build())
        .plugin(tauri_plugin_store::Builder::default().build());
        NApp { builder }
    }

    pub fn run(self) {
        self.builder.build(tauri::generate_context!()).unwrap().run(
            |_app_handle, event| match event {
                tauri::RunEvent::ExitRequested { api, .. } => {
                    api.prevent_exit();
                }
                _ => {}
            },
        );
    }

    pub fn register_module<T>(mut self, state: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.builder = self.builder.manage(state);
        return self;
    }
}

pub fn lmian() {
    NApp::new().run()
}