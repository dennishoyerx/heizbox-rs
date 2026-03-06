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
}

impl Default for HeaterSettings {
    fn default() -> Self {
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
    }
}
