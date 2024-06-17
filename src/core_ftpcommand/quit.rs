use tokio::io::{AsyncWriteExt, Result};

pub async fn handle_quit_command<W: AsyncWriteExt + Unpin>(writer: &mut W) -> Result<()> {
    // Handle the QUIT command
    writer.write_all(b"221 Service closing control connection.\r\n").await?;
    Ok(())
}
