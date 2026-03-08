use heizbox_hal::timer::{TimerDriver, TimerError};

pub struct TimerImpl {
    #[cfg(not(target_os = "espidf"))]
    start: std::time::Instant,
}

impl TimerImpl {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "espidf"))]
            start: std::time::Instant::now(),
        }
    }
}

impl TimerDriver for TimerImpl {
    fn start(&mut self, _channel: u8, _timeout_ms: u32) -> Result<(), TimerError> { Ok(()) }
    fn stop(&mut self, _channel: u8) -> Result<(), TimerError> { Ok(()) }
    fn is_running(&self, _channel: u8) -> bool { true }

    /// HAL-T4 ✅
    fn elapsed_ms(&self, _channel: u8) -> u64 {
        #[cfg(target_os = "espidf")]
        unsafe { esp_idf_sys::esp_timer_get_time() as u64 / 1000 }
        #[cfg(not(target_os = "espidf"))]
        self.start.elapsed().as_millis() as u64
    }
}
