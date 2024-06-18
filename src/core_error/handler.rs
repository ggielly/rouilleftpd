use anyhow::Result;
use crate::core_log::logger::log_message;

pub fn log_and_convert<E>(message: &str, error: E) -> Result<()>
where
    E: std::error::Error + Send + Sync + 'static,
{
    log_message(&format!("{}: {}", message, error));
    Err(anyhow::Error::new(error))
}
