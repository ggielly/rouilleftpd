use tokio::io::{AsyncWriteExt, Result};

pub async fn handle_user_command<W: AsyncWriteExt + Unpin>(writer: &mut W, _username: &str) -> Result<()> {
    // Handle the USER command
    // For simplicity, we just accept any username
    writer.write_all(b"331 User name okay, need password.\r\n").await?;
    Ok(())
}
