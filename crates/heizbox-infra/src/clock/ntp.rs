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
}
