use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimerError {
    #[error("Timer already running")]
    AlreadyRunning,
    #[error("Timer not running")]
    NotRunning,
    #[error("Invalid channel")]
    InvalidChannel,
}

pub trait TimerDriver: Send + Sync {
    fn start(&mut self, channel: u8, timeout_ms: u32) -> Result<(), TimerError>;
    fn stop(&mut self, channel: u8) -> Result<(), TimerError>;
    fn is_running(&self, channel: u8) -> bool;
}
