use tokio::io::{AsyncWriteExt, Result};

pub async fn handle_pass_command<W: AsyncWriteExt + Unpin>(writer: &mut W, _password: &str) -> Result<()> {
    // Handle the PASS command
    // For simplicity, we just accept any password
    writer.write_all(b"230 User logged in, proceed.\r\n").await?;
    Ok(())
}
