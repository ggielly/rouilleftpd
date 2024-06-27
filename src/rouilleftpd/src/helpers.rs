use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Sanitizes input to prevent directory traversal attacks.
///
/// # Arguments
///
/// * `input` - The input string to sanitize.
///
/// # Returns
///
/// A sanitized string.
pub fn sanitize_input(input: &str) -> String {
    input.replace("../", "").replace("..\\", "")
}

/// Sends a response to the client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `message` - The message to send.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn send_response(
    writer: &Arc<Mutex<TcpStream>>,
    message: &[u8],
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(message).await?;
    Ok(())
}
