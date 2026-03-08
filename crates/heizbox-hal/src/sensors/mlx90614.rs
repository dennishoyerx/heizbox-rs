use crate::{I2cDriver, I2cError};

/// MLX90614 IR temperature sensor driver (SMBus Read-Word protocol).
/// Default 7-bit I²C address: 0x5A.
pub struct Mlx90614 {
    i2c: Box<dyn I2cDriver + Send>,
    address: u8,
}

impl Mlx90614 {
    pub fn new(i2c: Box<dyn I2cDriver + Send>, address: u8) -> Self {
        Self { i2c, address }
    }

    fn read_word(&mut self, register: u8) -> Result<u16, I2cError> {
        let data = self.i2c.write_read(self.address, &[register], 2)?;
        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    pub fn read_object_temp(&mut self) -> Result<u16, I2cError> {
        let raw = self.read_word(0x06)?;
        Ok(((raw as f32 * 0.02) - 273.15).round() as u16)
    }

    pub fn read_ambient_temp(&mut self) -> Result<u16, I2cError> {
        let raw = self.read_word(0x07)?;
        Ok(((raw as f32 * 0.02) - 273.15).round() as u16)
    }

    /// Returns (object_temp_°C, ambient_temp_°C, raw_ir_word).
    pub fn read_all(&mut self) -> Result<(u16, u16, u16), I2cError> {
        let obj_raw = self.read_word(0x06)?;
        let amb_raw = self.read_word(0x07)?;
        let obj_c   = ((obj_raw as f32 * 0.02) - 273.15).round() as u16;
        let amb_c   = ((amb_raw as f32 * 0.02) - 273.15).round() as u16;
        Ok((obj_c, amb_c, obj_raw))
    }
}
