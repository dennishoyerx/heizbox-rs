/// Manages system time, synced via NTP on startup.
pub struct ClockManager {
    /// UNIX timestamp offset: real_time = esp_timer_get_time() / 1_000_000 + offset_secs
    offset_secs: i64,
    synced: bool,
}

impl ClockManager {
    pub fn new() -> Self {
        Self {
            offset_secs: 0,
            synced: false,
        }
    }

    /// Apply an NTP-provided offset.
    pub fn set_offset(&mut self, offset_secs: i64) {
        self.offset_secs = offset_secs;
        self.synced = true;
    }

    /// Returns `true` once the clock has been synchronised.
    pub fn is_synced(&self) -> bool {
        self.synced
    }

    /// Returns the current UNIX timestamp (requires `esp_timer_get_time` in the
    /// ESP32 implementation; returns 0 here as a stub).
    pub fn now_unix(&self) -> i64 {
        self.offset_secs
    }
}

impl Default for ClockManager {
    fn default() -> Self {
        Self::new()
    }
}
