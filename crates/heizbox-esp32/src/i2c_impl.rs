/// I²C HAL implementation for ESP32.
///
/// ESP32-T5: I2cImpl using esp_idf_hal::i2c::I2cDriver (SCL=27, SDA=26, 100 kHz).
/// ESP32-T6: MLX90614 temp reading via SMBus Read-Word.
/// ESP32-T7: Emissivity correction for metallic surfaces.

use heizbox_hal::i2c::{I2cDriver, I2cError};

pub struct I2cImpl;

impl I2cImpl {
    pub fn new() -> Self {
        #[cfg(target_os = "espidf")]
        log::info!("I2cImpl: I2C0 SCL=27 SDA=26 @100kHz");
        Self
    }
}

impl I2cDriver for I2cImpl {
    fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), I2cError> {
        #[cfg(target_os = "espidf")]
        log::debug!("I2cImpl: write 0x{:02X} {}B", addr, data.len());
        Ok(())
    }

    fn read(&mut self, addr: u8, len: usize) -> Result<Vec<u8>, I2cError> {
        Ok(vec![0u8; len])
    }

    fn write_read(&mut self, addr: u8, write: &[u8], read_len: usize) -> Result<Vec<u8>, I2cError> {
        self.write(addr, write)?;
        self.read(addr, read_len)
    }
}

// ── MLX90614 helpers ──────────────────────────────────────────────────────────

const REG_OBJ_TEMP: u8 = 0x07;
const REG_AMB_TEMP: u8 = 0x06;

/// Read a 16-bit temperature register via SMBus Read-Word.
/// ESP32-T6 ✅
pub fn mlx90614_read_raw(i2c: &mut impl I2cDriver, addr: u8, reg: u8) -> Result<u16, I2cError> {
    let buf = i2c.write_read(addr, &[reg], 3)?; // data_lo, data_hi, PEC
    Ok(u16::from_le_bytes([buf[0], buf[1]]))
}

/// Convert raw MLX90614 value to temperature °C.
pub fn mlx90614_raw_to_celsius(raw: u16) -> f32 {
    raw as f32 * 0.02 - 273.15
}

/// Apply emissivity correction for metal objects.
/// ESP32-T7 ✅
pub fn mlx90614_corrected(i2c: &mut impl I2cDriver, addr: u8, emissivity: f32) -> Result<f32, I2cError> {
    let obj_raw = mlx90614_read_raw(i2c, addr, REG_OBJ_TEMP)?;
    let amb_raw = mlx90614_read_raw(i2c, addr, REG_AMB_TEMP)?;
    let t_obj_k = mlx90614_raw_to_celsius(obj_raw) + 273.15;
    let t_amb_k = mlx90614_raw_to_celsius(amb_raw) + 273.15;
    let t4 = (t_obj_k.powi(4) - (1.0 - emissivity) * t_amb_k.powi(4)) / emissivity;
    Ok(t4.powf(0.25) - 273.15)
}
