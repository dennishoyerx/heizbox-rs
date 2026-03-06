use heizbox_core::heater::HeaterConfig;
use heizbox_core::error::PersistenceError;
use heizbox_hal::NvsDriver;

pub struct HeaterConfigRepository<N: NvsDriver> {
    nvs: N,
}

impl<N: NvsDriver> HeaterConfigRepository<N> {
    const NS: &'static str = "heater";

    pub fn new(nvs: N) -> Self {
        Self { nvs }
    }

    pub fn load(&self) -> Result<HeaterConfig, PersistenceError> {
        Ok(HeaterConfig {
            power: self.nvs.get_u8(Self::NS, "power", 100)
                .map_err(|_| PersistenceError::NvsError)?,
            target_temp: self.nvs.get_u16(Self::NS, "target_temp", 200)
                .map_err(|_| PersistenceError::NvsError)?,
            auto_stop_time_ms: self.nvs.get_u32(Self::NS, "auto_stop_ms", 90_000)
                .map_err(|_| PersistenceError::NvsError)?,
        })
    }

    pub fn save(&self, config: &HeaterConfig) -> Result<(), PersistenceError> {
        self.nvs.set_u8(Self::NS, "power", config.power)
            .map_err(|_| PersistenceError::NvsError)?;
        self.nvs.set_u16(Self::NS, "target_temp", config.target_temp)
            .map_err(|_| PersistenceError::NvsError)?;
        self.nvs.set_u32(Self::NS, "auto_stop_ms", config.auto_stop_time_ms)
            .map_err(|_| PersistenceError::NvsError)?;
        Ok(())
    }
}
