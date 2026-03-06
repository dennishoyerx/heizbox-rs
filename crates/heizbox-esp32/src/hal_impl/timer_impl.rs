use heizbox_hal::{TimerDriver, TimerError};

pub struct TimerImpl;

impl TimerImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TimerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerDriver for TimerImpl {
    fn start(&mut self, _channel: u8, _timeout_ms: u32) -> Result<(), TimerError> {
        Ok(())
    }

    fn stop(&mut self, _channel: u8) -> Result<(), TimerError> {
        Ok(())
    }

    fn is_running(&self, _channel: u8) -> bool {
        false
    }
}
