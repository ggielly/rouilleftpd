use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use std::io;

pub async fn handle_list_command(writer: Arc<Mutex<TcpStream>>, _config: Arc<Config>, _arg: String) -> Result<(), io::Error> {
    // Implement the logic for the LIST command here
    let mut writer = writer.lock().await;
    writer.write_all(b"150 Opening ASCII mode data connection for file list.\r\n").await?;
    // Example response, you would generate the actual directory listing here
    writer.write_all(b"-rw-r--r-- 1 owner group 2134 Jan 01 00:00 file1.txt\r\n").await?;
    writer.write_all(b"-rw-r--r-- 1 owner group 523 Jan 01 00:00 file2.txt\r\n").await?;
    writer.write_all(b"226 Transfer complete.\r\n").await?;
    Ok(())
}
