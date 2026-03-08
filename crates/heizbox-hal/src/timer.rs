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

/// HAL-T4: `elapsed_ms` lets callers query running time without stopping the timer.
pub trait TimerDriver: Send + Sync {
    fn start(&mut self, channel: u8, timeout_ms: u32) -> Result<(), TimerError>;
    fn stop(&mut self, channel: u8) -> Result<(), TimerError>;
    fn is_running(&self, channel: u8) -> bool;
    /// Elapsed milliseconds since the timer was last started on `channel`.
    /// Returns `0` if the timer has never been started or is not running.
    fn elapsed_ms(&self, channel: u8) -> u64;
}
