use crate::core_network::Session;
use crate::Config;
use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the TYPE FTP command.
///
/// This function sets the transfer mode to either ASCII, EBCDIC, Binary, or Local Byte.
/// It sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The argument specifying the transfer type.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_type_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    let parts: Vec<&str> = arg.split_whitespace().collect();
    let primary_type = parts.get(0).map(|s| s.to_uppercase()).unwrap_or_default();
    let second_arg = parts.get(1).map(|s| s.to_string());

    let response = match primary_type.as_str() {
        "A" => {
            let mut session = session.lock().await;
            session.type_ = "A".to_string();
            session.byte_size = None;
            "200 Type set to A\r\n".to_string()
        }
        "E" => {
            let mut session = session.lock().await;
            session.type_ = "E".to_string();
            session.byte_size = None;
            "200 Type set to E\r\n".to_string()
        }
        "I" => {
            let mut session = session.lock().await;
            session.type_ = "I".to_string();
            session.byte_size = None;
            "200 Type set to I\r\n".to_string()
        }
        "L" => {
            if let Some(size) = second_arg {
                match size.parse::<u8>() {
                    Ok(byte_size) => {
                        let mut session = session.lock().await;
                        session.type_ = "L".to_string();
                        session.byte_size = Some(byte_size);
                        format!("200 Type set to L ({})\r\n", byte_size)
                    }
                    Err(_) => "504 Invalid byte size parameter.\r\n".to_string(),
                }
            } else {
                "504 Byte size parameter required for TYPE L.\r\n".to_string()
            }
        }
        _ => "504 Command not implemented for that parameter.\r\n".to_string(),
    };

    let mut writer = writer.lock().await;
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}
