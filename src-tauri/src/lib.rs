pub mod command;
mod client;
mod plugin;
use log::LevelFilter;
use tauri::Wry;

use command::generate_unique_message_id;
use tauri_plugin_log::{LogTarget};

enum LogLevel {
    // 定义你的日志级别
    Info,
    // 其他日志级别
}

// 将自定义的 LogLevel 转换为 log::LevelFilter
impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Info => LevelFilter::Info,
            // 其他日志级别的映射
        }
    }
}

pub struct NApp {
    builder: tauri::Builder<Wry>,
}

impl NApp {
    pub fn new() -> NApp {
        // 在你的代码中使用 LogLevel::Info 并进行转换
        let log_level = LogLevel::Info;
        let level_filter: LevelFilter = log_level.into();
        let builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![generate_unique_message_id])
        .plugin(tauri_plugin_log::Builder::default().targets([
            LogTarget::LogDir,
            LogTarget::Stdout,
            LogTarget::Webview,
        ]).level(level_filter).build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(plugin::connect::Builder::default().build());
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