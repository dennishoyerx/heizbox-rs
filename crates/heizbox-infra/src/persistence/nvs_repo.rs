//! NVS repository implementations.
//!
//! NVS namespace strategy (INFRA-T11 ✅):
//!   "htr_cfg" → HeaterConfigRepository
//!   "htr_set" → HeaterSettingsRepository

use heizbox_core::heater::state::HeaterConfig;
use heizbox_core::consumption::ConsumptionData;
use heizbox_core::error::PersistenceError;
use heizbox_core::heating_mode::Preset;
use heizbox_hal::nvs::{NvsDriver, NvsError};
use super::models::{HeaterSettings, StoredHeaterConfig, StoredPreset};

const NS_CFG: &str = "htr_cfg";
const NS_SET: &str = "htr_set";
const PRESET_SLOTS: usize = 4;

// ── HeaterConfigRepository ────────────────────────────────────────────────────

pub struct HeaterConfigRepository<N: NvsDriver> { nvs: N }

impl<N: NvsDriver> HeaterConfigRepository<N> {
    pub fn new(nvs: N) -> Self { Self { nvs } }

    /// INFRA-T9 ✅ Load config; falls back to defaults on missing keys.
    pub fn load(&mut self) -> Result<HeaterConfig, PersistenceError> {
        let target_temp = self.get_u16("target_temp")?
            .unwrap_or(heizbox_core::config::DEFAULT_TARGET_TEMP);
        let auto_stop_ms = self.get_u32("auto_stop_ms")?
            .unwrap_or(heizbox_core::config::DEFAULT_AUTO_STOP_MS);
        let power = self.get_u8("power")?
            .unwrap_or(heizbox_core::config::DEFAULT_POWER);
        Ok(HeaterConfig { target_temp, auto_stop_time_ms: auto_stop_ms, power })
    }

    /// INFRA-T9 ✅ Persist config.
    pub fn save(&mut self, config: &HeaterConfig) -> Result<(), PersistenceError> {
        self.set_u16("target_temp", config.target_temp)?;
        self.set_u32("auto_stop_ms", config.auto_stop_time_ms)?;
        self.set_u8("power", config.power)
    }

    fn get_u8(&mut self, key: &str) -> Result<Option<u8>, PersistenceError> {
        match self.nvs.get_u8(NS_CFG, key) {
            Ok(v) => Ok(v),
            Err(NvsError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(PersistenceError::from(e)),
        }
    }
    fn get_u16(&mut self, key: &str) -> Result<Option<u16>, PersistenceError> {
        match self.nvs.get_u16(NS_CFG, key) {
            Ok(v) => Ok(v),
            Err(NvsError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(PersistenceError::from(e)),
        }
    }
    fn get_u32(&mut self, key: &str) -> Result<Option<u32>, PersistenceError> {
        match self.nvs.get_u32(NS_CFG, key) {
            Ok(v) => Ok(v),
            Err(NvsError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(PersistenceError::from(e)),
        }
    }
    fn set_u8(&mut self, key: &str, v: u8) -> Result<(), PersistenceError> {
        self.nvs.set_u8(NS_CFG, key, v).map_err(Into::into)
    }
    fn set_u16(&mut self, key: &str, v: u16) -> Result<(), PersistenceError> {
        self.nvs.set_u16(NS_CFG, key, v).map_err(Into::into)
    }
    fn set_u32(&mut self, key: &str, v: u32) -> Result<(), PersistenceError> {
        self.nvs.set_u32(NS_CFG, key, v).map_err(Into::into)
    }
}

// ── HeaterSettingsRepository ──────────────────────────────────────────────────

pub struct HeaterSettingsRepository<N: NvsDriver> { nvs: N }

impl<N: NvsDriver> HeaterSettingsRepository<N> {
    pub fn new(nvs: N) -> Self { Self { nvs } }

    /// INFRA-T10 ✅ Load settings; missing keys treated as zero.
    pub fn load_settings(&mut self) -> Result<HeaterSettings, PersistenceError> {
        let total_cycles = self.get_u32("total_cycles")?.unwrap_or(0);
        let dur_lo       = self.get_u32("total_dur_lo")?.unwrap_or(0) as u64;
        let dur_hi       = self.get_u32("total_dur_hi")?.unwrap_or(0) as u64;
        let screensaver_timeout_s = self.get_u32("ss_timeout_s")?.unwrap_or(60);
        Ok(HeaterSettings {
            total_cycles,
            total_duration_ms: (dur_hi << 32) | dur_lo,
            screensaver_timeout_s,
        })
    }

    /// INFRA-T10 ✅ Persist ConsumptionData totals.
    pub fn save_consumption(&mut self, data: &ConsumptionData) -> Result<(), PersistenceError> {
        self.set_u32("total_cycles", data.total_cycles)?;
        self.set_u32("total_dur_lo", (data.total_duration_ms & 0xFFFF_FFFF) as u32)?;
        self.set_u32("total_dur_hi", (data.total_duration_ms >> 32) as u32)
    }

    /// INFRA-T12 ✅ Load preset slot (0..3).
    pub fn load_preset(&mut self, slot: usize) -> Result<Option<(u16, u8)>, PersistenceError> {
        assert!(slot < PRESET_SLOTS);
        let key = preset_key(slot);
        match self.nvs.get_blob(NS_SET, key.as_str()) {
            Ok(Some(b)) if b.len() >= 3 => Ok(Some(StoredPreset::from_bytes([b[0],b[1],b[2]]))),
            Ok(_) => Ok(None),
            Err(NvsError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// INFRA-T12 ✅ Save preset slot (0..3).
    pub fn save_preset(&mut self, slot: usize, preset: Preset,
                        ot: Option<u16>, op: Option<u8>) -> Result<(), PersistenceError> {
        assert!(slot < PRESET_SLOTS);
        let key = preset_key(slot);
        let bytes = StoredPreset { preset, override_target: ot, override_power: op }.to_bytes();
        self.nvs.set_blob(NS_SET, key.as_str(), &bytes).map_err(Into::into)
    }

    fn get_u32(&mut self, key: &str) -> Result<Option<u32>, PersistenceError> {
        match self.nvs.get_u32(NS_SET, key) {
            Ok(v) => Ok(v),
            Err(NvsError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    fn set_u32(&mut self, key: &str, v: u32) -> Result<(), PersistenceError> {
        self.nvs.set_u32(NS_SET, key, v).map_err(Into::into)
    }
}

fn preset_key(slot: usize) -> heapless::String<16> {
    use core::fmt::Write;
    let mut s = heapless::String::new();
    let _ = write!(s, "preset_{}", slot);
    s
}
