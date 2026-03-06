use async_trait::async_trait;
use heizbox_hal::{I2cDriver, I2cError};

/// ESP32 I2C stub.
/// Replace with real `esp_idf_hal::i2c::I2cDriver` calls in production.
pub struct I2cImpl;

impl I2cImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for I2cImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl I2cDriver for I2cImpl {
    async fn write(&mut self, _addr: u8, _data: &[u8]) -> Result<(), I2cError> {
        Ok(())
    }

    async fn read(&mut self, _addr: u8, len: usize) -> Result<Vec<u8>, I2cError> {
        Ok(vec![0u8; len])
    }

    async fn write_read(
        &mut self,
        _addr: u8,
        _write: &[u8],
        read_len: usize,
    ) -> Result<Vec<u8>, I2cError> {
        Ok(vec![0u8; read_len])
    }
}
