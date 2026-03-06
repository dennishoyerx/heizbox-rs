use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::sys::{self, EspError};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Manages system time, synced via NTP on startup.
pub struct ClockManager {
    /// UNIX timestamp offset: real_time = boot_time_secs + offset_secs
    offset_secs: i64,
    synced: bool,
}

#[derive(Debug, Error)]
pub enum ClockError {
    #[error("SNTP error: {0}")]
    Sntp(#[from] EspError),
    #[error("Timeout waiting for NTP sync")]
    Timeout,
    #[error("System time not available")]
    SystemTimeUnavailable,
}

impl ClockManager {
    pub fn new() -> Self {
        Self {
            offset_secs: 0,
            synced: false,
        }
    }

    /// Synchronize the clock with an NTP server.
    ///
    /// This initializes the SNTP client, waits for synchronization, and
    /// computes the offset between the ESP32's boot timer and UTC.
    ///
    /// Should be called after the WiFi connection is established.
    pub fn sync_ntp(&mut self) -> Result<(), ClockError> {
        // Create SNTP client with default configuration (uses built-in NTP servers)
        let sntp = EspSntp::new_default()?;

        // Wait for synchronization with a timeout (10 seconds)
        let timeout = Duration::from_secs(10);
        let start = Instant::now();
        loop {
            match sntp.get_sync_status() {
                SyncStatus::Completed => break,
                SyncStatus::Reset | SyncStatus::InProgress => {
                    if start.elapsed() >= timeout {
                        return Err(ClockError::Timeout);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }

        // Get current NTP time (system time after sync)
        let now_duration = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_err(|_| ClockError::SystemTimeUnavailable)?;
        let ntp_secs = now_duration.as_secs() as i64;

        // Get current boot time in seconds
        let boot_us = unsafe { sys::esp_timer_get_time() };
        let boot_secs = boot_us as i64 / 1_000_000;

        // Compute offset: offset = ntp_time - boot_time
        self.offset_secs = ntp_secs - boot_secs;
        self.synced = true;

        // SNTP client is dropped here, stopping the service (one-shot sync)
        Ok(())
    }

    /// Apply an NTP-provided offset directly (for use by external sync mechanisms).
    pub fn set_offset(&mut self, offset_secs: i64) {
        self.offset_secs = offset_secs;
        self.synced = true;
    }

    /// Returns `true` once the clock has been synchronised.
    pub fn is_synced(&self) -> bool {
        self.synced
    }

    /// Returns the current UNIX timestamp (seconds since Unix epoch).
    ///
    /// If the clock has been synchronized via NTP, this returns an accurate
    /// UTC timestamp. Otherwise, returns the boot time plus any manually set offset.
    pub fn now_unix(&self) -> i64 {
        let boot_us = unsafe { sys::esp_timer_get_time() };
        let boot_secs = boot_us as i64 / 1_000_000;
        boot_secs + self.offset_secs
    }
}

impl Default for ClockManager {
    fn default() -> Self {
        Self::new()
    }
}
