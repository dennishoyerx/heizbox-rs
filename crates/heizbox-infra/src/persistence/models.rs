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
}

impl Default for HeaterSettings {
    fn default() -> Self {
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
    }
}
