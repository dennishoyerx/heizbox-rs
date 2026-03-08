<<<<<<< ours
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
=======
/// NTP-based clock synchronisation.
///
/// INFRA-T16: ClockManager::sync_ntp() using esp_idf_svc::sntp::EspSntp.
/// INFRA-T17: Fallback to last-known timestamp from NVS when NTP unreachable.

use heizbox_core::config::NTP_SERVER;
use heizbox_hal::nvs::NvsDriver;
use heizbox_core::error::PersistenceError;

const NS_CLK: &str = "clock";
const KEY_TS: &str  = "last_ts";

/// Thin wrapper around the SNTP service.
///
/// On `no_std` / ESP-IDF targets this calls `EspSntp` via the
/// `esp_idf_svc` crate.  In host tests the struct is a no-op shim.
pub struct ClockManager<N: NvsDriver> {
    nvs: N,
    /// Monotonic epoch offset so `now_ms()` returns wall-clock time even
    /// before NTP succeeds.
    epoch_offset_ms: u64,
}

impl<N: NvsDriver> ClockManager<N> {
    pub fn new(nvs: N) -> Self {
        Self { nvs, epoch_offset_ms: 0 }
    }

    /// Synchronise time via SNTP.
    ///
    /// On success the internal epoch offset is updated and the timestamp is
    /// persisted to NVS (INFRA-T17 fallback store).
    ///
    /// On ESP-IDF targets this blocks until the SNTP callback fires or the
    /// timeout (10 s) elapses.
    pub fn sync_ntp(&mut self) -> Result<u64, ClockError> {
        #[cfg(target_os = "espidf")]
        {
            use esp_idf_svc::sntp::{EspSntp, SyncStatus};
            use std::time::Duration;
            use esp_idf_hal::delay::FreeRtos;

            let sntp = EspSntp::new_default().map_err(|e| ClockError::SntpInit(e.to_string()))?;
            // Wait up to 10 s for sync.
            let deadline = 10_000u32;
            let mut elapsed = 0u32;
            while sntp.get_sync_status() != SyncStatus::Completed {
                if elapsed >= deadline {
                    return self.load_fallback_ts();
                }
                FreeRtos::delay_ms(200);
                elapsed += 200;
            }
            let ts = unix_time_ms();
            self.epoch_offset_ms = ts;
            let _ = self.persist_ts(ts);
            Ok(ts)
        }
        #[cfg(not(target_os = "espidf"))]
        {
            // Host / test shim — return a fixed timestamp.
            let ts = 1_700_000_000_000u64;
            self.epoch_offset_ms = ts;
            Ok(ts)
        }
    }

    /// INFRA-T17: Load last persisted timestamp from NVS as fallback.
    pub fn load_fallback_ts(&mut self) -> Result<u64, ClockError> {
        let lo = self.nvs.get_u32(NS_CLK, "ts_lo")
            .unwrap_or(None).unwrap_or(0) as u64;
        let hi = self.nvs.get_u32(NS_CLK, "ts_hi")
            .unwrap_or(None).unwrap_or(0) as u64;
        let ts = (hi << 32) | lo;
        self.epoch_offset_ms = ts;
        Ok(ts)
    }

    /// Persist timestamp to NVS (split u64 → two u32 keys).
    fn persist_ts(&mut self, ts: u64) -> Result<(), PersistenceError> {
        self.nvs.set_u32(NS_CLK, "ts_lo", (ts & 0xFFFF_FFFF) as u32)?;
        self.nvs.set_u32(NS_CLK, "ts_hi", (ts >> 32) as u32)?;
        Ok(())
    }

    /// Current wall-clock time in milliseconds since Unix epoch.
    /// Returns best-effort value using the stored offset.
    pub fn now_ms(&self) -> u64 {
        #[cfg(target_os = "espidf")]
        { unix_time_ms() }
        #[cfg(not(target_os = "espidf"))]
        { self.epoch_offset_ms }
    }
}

#[cfg(target_os = "espidf")]
fn unix_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ClockError {
    SntpInit(String),
    NvsFallback,
>>>>>>> theirs
}
