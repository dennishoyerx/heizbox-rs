use super::models::{HeaterSettings, HeatingMode};
use heizbox_core::error::PersistenceError;
use heizbox_hal::NvsDriver;

pub struct HeaterSettingsRepository<N: NvsDriver> {
    nvs: N,
}

impl<N: NvsDriver> HeaterSettingsRepository<N> {
    const NS: &'static str = "heater";

    pub fn new(nvs: N) -> Self {
        Self { nvs }
    }

    pub fn load(&self) -> Result<HeaterSettings, PersistenceError> {
        let e = |_| PersistenceError::NvsError;
        Ok(HeaterSettings {
            power: self.nvs.get_u8(Self::NS, "power", 100).map_err(e)?,
            mode: HeatingMode::Preset,
            presets: [
                self.nvs.get_u16(Self::NS, "preset_0", 185).map_err(e)?,
                self.nvs.get_u16(Self::NS, "preset_1", 200).map_err(e)?,
                self.nvs.get_u16(Self::NS, "preset_2", 210).map_err(e)?,
                self.nvs.get_u16(Self::NS, "preset_3", 220).map_err(e)?,
            ],
            ir_emissivity: self.nvs.get_u8(Self::NS, "ir_emissivity", 95).map_err(e)?,
            ir_correction: self.nvs.get_i32(Self::NS, "ir_correction", 0).map_err(e)? as i8,
            temp_sensor_read_interval_ms: self.nvs.get_u16(Self::NS, "read_interval", 220).map_err(e)?,
            temp_sensor_off_time_ms: self.nvs.get_u16(Self::NS, "off_time", 0).map_err(e)?,
        })
    }

    pub fn save(&self, s: &HeaterSettings) -> Result<(), PersistenceError> {
        let e = |_| PersistenceError::NvsError;
        self.nvs.set_u8(Self::NS, "power", s.power).map_err(e)?;
        for (i, &p) in s.presets.iter().enumerate() {
            let key = match i {
                0 => "preset_0", 1 => "preset_1", 2 => "preset_2", _ => "preset_3",
            };
            self.nvs.set_u16(Self::NS, key, p).map_err(e)?;
        }
        self.nvs.set_u8(Self::NS, "ir_emissivity", s.ir_emissivity).map_err(e)?;
        Ok(())
    }
}
