use heizbox_hal::{AdcDriver, AdcError};

/// ESP32 ADC stub — always returns 0.
/// Replace with `esp_idf_hal::adc::AdcDriver` calls in production.
pub struct AdcImpl;

impl AdcImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AdcImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AdcDriver for AdcImpl {
    fn read(&self, _pin: u8) -> Result<u16, AdcError> {
        Ok(0)
    }
}
