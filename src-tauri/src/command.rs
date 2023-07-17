use std::{path::Path, fs::File};
use uuid::Uuid;

#[tauri::command]
pub fn generate_unique_message_id() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}


// 创建文件
#[tauri::command]
pub fn create_file(path: String) -> Result<(), String> {
    let path = Path::new(&path);
    let display = path.display();
    match File::create(&path) {
        Err(why) => return Err(format!("couldn't create {}: {}", display, why)),
        Ok(file) => file,
    };
    Ok(())
}
