<<<<<<< ours
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HeatingMode {
    Temperature,
    Preset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaterSettings {
    pub power: u8,
    pub mode: HeatingMode,
    /// Temperature presets: [Flavour, Balanced, Extraction, Full].
    pub presets: [u16; 4],
    pub ir_emissivity: u8,
    pub ir_correction: i8,
    pub temp_sensor_read_interval_ms: u16,
    pub temp_sensor_off_time_ms: u16,
=======
use heizbox_core::heater::state::HeaterConfig;
use heizbox_core::heating_mode::Preset;

/// Flat NVS representation of HeaterConfig.
/// Namespace: "htr_cfg"
#[derive(Debug, Clone)]
pub struct StoredHeaterConfig {
    pub target_temp: u16,    // key "target_temp"
    pub auto_stop_ms: u32,   // key "auto_stop_ms"
    pub power: u8,           // key "power"
}

impl From<&HeaterConfig> for StoredHeaterConfig {
    fn from(c: &HeaterConfig) -> Self {
        Self { target_temp: c.target_temp, auto_stop_ms: c.auto_stop_time_ms, power: c.power }
    }
}

impl From<StoredHeaterConfig> for HeaterConfig {
    fn from(s: StoredHeaterConfig) -> Self {
        HeaterConfig { target_temp: s.target_temp, auto_stop_time_ms: s.auto_stop_ms, power: s.power }
    }
}

/// Persisted device stats. Namespace: "htr_set"
#[derive(Debug, Clone)]
pub struct HeaterSettings {
    pub total_cycles: u32,
    pub total_duration_ms: u64,
    pub screensaver_timeout_s: u32,
>>>>>>> theirs
}

impl Default for HeaterSettings {
    fn default() -> Self {
<<<<<<< ours
        Self {
            power: 100,
            mode: HeatingMode::Preset,
            presets: [185, 200, 210, 220],
            ir_emissivity: 95,
            ir_correction: 0,
            temp_sensor_read_interval_ms: 220,
            temp_sensor_off_time_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub brightness: u8,
    pub idle_brightness: u8,
    pub idle_timeout_minutes: u16,
    pub dark_mode: bool,
    pub flip_orientation: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            brightness: 100,
            idle_brightness: 30,
            idle_timeout_minutes: 5,
            dark_mode: false,
            flip_orientation: false,
        }
=======
        Self { total_cycles: 0, total_duration_ms: 0, screensaver_timeout_s: 60 }
    }
}

/// 3-byte packed preset: [target_lo, target_hi, power]
#[derive(Debug, Clone, Copy)]
pub struct StoredPreset {
    pub preset: Preset,
    pub override_target: Option<u16>,
    pub override_power: Option<u8>,
}

impl StoredPreset {
    pub fn to_bytes(self) -> [u8; 3] {
        let t = self.override_target.unwrap_or(self.preset.target_temp());
        let p = self.override_power.unwrap_or(self.preset.power());
        [(t & 0xFF) as u8, (t >> 8) as u8, p]
    }
    pub fn from_bytes(b: [u8; 3]) -> (u16, u8) {
        (u16::from_le_bytes([b[0], b[1]]), b[2])
>>>>>>> theirs
    }
}
