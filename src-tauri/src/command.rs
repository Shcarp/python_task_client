use uuid::Uuid;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn generate_unique_message_id() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

