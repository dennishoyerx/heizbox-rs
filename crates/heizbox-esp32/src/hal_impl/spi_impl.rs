use heizbox_hal::{SpiDriver, SpiError};

/// ESP32 SPI stub.
/// Replace with real `esp_idf_hal::spi::SpiDeviceDriver` calls in production.
pub struct SpiImpl;

impl SpiImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpiImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SpiDriver for SpiImpl {
    fn write(&mut self, _data: &[u8]) -> Result<(), SpiError> {
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<(), SpiError> {
        buffer.fill(0);
        Ok(())
    }

    fn transfer(&mut self, _write: &[u8], read: &mut [u8]) -> Result<(), SpiError> {
        read.fill(0);
        Ok(())
    }
}
