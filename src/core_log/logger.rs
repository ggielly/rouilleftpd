use chrono::Local;

pub fn log_message(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("[{}] {}", timestamp, message);
}
