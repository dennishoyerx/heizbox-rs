use heizbox_hal::{GpioDriver, GpioError};

/// ESP32 GPIO stub.
/// Replace with real `esp_idf_hal::gpio::PinDriver` calls in production.
pub struct GpioImpl {
    /// (pin, is_high)
    state: std::collections::HashMap<u8, bool>,
}

impl GpioImpl {
    pub fn new() -> Self {
        Self {
            state: std::collections::HashMap::new(),
        }
    }
}

impl Default for GpioImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl GpioDriver for GpioImpl {
    fn set_output(&mut self, pin: u8) -> Result<(), GpioError> {
        if pin > 39 {
            return Err(GpioError::InvalidPin(pin));
        }
        self.state.entry(pin).or_insert(false);
        Ok(())
    }

    fn set_input(&mut self, pin: u8) -> Result<(), GpioError> {
        if pin > 39 {
            return Err(GpioError::InvalidPin(pin));
        }
        self.state.entry(pin).or_insert(false);
        Ok(())
    }

    fn write(&mut self, pin: u8, high: bool) -> Result<(), GpioError> {
        self.state.insert(pin, high);
        Ok(())
    }

    fn read(&self, pin: u8) -> Result<bool, GpioError> {
        self.state.get(&pin).copied().ok_or(GpioError::NotConfigured)
    }
}
