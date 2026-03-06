use crate::{I2cDriver, I2cError};

/// MLX90614 IR temperature sensor driver.
///
/// Uses SMBus Read-Word protocol.
/// Default I2C address is 0x5A (7-bit).
pub struct Mlx90614 {
    i2c: Box<dyn I2cDriver + Send>,
    address: u8,
}

impl Mlx90614 {
    /// Create a new MLX90614 driver.
    /// `i2c` must be a boxed dyn I2cDriver (to allow trait object).
    /// `address` is the 7-bit I2C address (default 0x5A).
    pub fn new(i2c: Box<dyn I2cDriver + Send>, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Read a 16-bit word from the given register.
    fn read_word(&mut self, register: u8) -> Result<u16, I2cError> {
        let data = self
            .i2c
            .write_read(self.address, &[register], 2)
            .map_err(|_| I2cError::BusError)?;
        // MLX90614 returns big-endian.
        let raw = u16::from_be_bytes([data[0], data[1]]);
        Ok(raw)
    }

    /// Read object (IR) temperature in Celsius (rounded to integer).
    pub fn read_object_temp(&mut self) -> Result<u16, I2cError> {
        let raw = self.read_word(0x06)?;
        // Convert: temp_c = (raw * 0.02) - 273.15
        let temp_c = (raw as f32 * 0.02) - 273.15;
        Ok(temp_c.round() as u16)
    }

    /// Read ambient (sensor die) temperature in Celsius (rounded to integer).
    pub fn read_ambient_temp(&mut self) -> Result<u16, I2cError> {
        let raw = self.read_word(0x07)?;
        let temp_c = (raw as f32 * 0.02) - 273.15;
        Ok(temp_c.round() as u16)
    }

    /// Read both temperatures at once.
    /// Returns (object_temp_celsius, ambient_temp_celsius, raw_ir_object).
    pub fn read_all(&mut self) -> Result<(u16, u16, u16), I2cError> {
        let object_raw = self.read_word(0x06)?;
        let ambient_raw = self.read_word(0x07)?;
        let object_c = (object_raw as f32 * 0.02) - 273.15;
        let ambient_c = (ambient_raw as f32 * 0.02) - 273.15;
        Ok((object_c.round() as u16, ambient_c.round() as u16, object_raw))
    }
}
