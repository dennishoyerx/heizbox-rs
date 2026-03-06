use esp_idf_hal::gpio::{Pin, InputPin, OutputPin};
use esp_idf_hal::i2c::{I2cDriver as EspI2cDriver, config::Config};
use esp_idf_hal::i2c::I2C0;
use heizbox_hal::{I2cDriver, I2cError};

/// ESP32 I2C driver using `esp_idf_hal`.
///
/// Pin configuration (from hardware spec):
/// - SDA: GPIO 26
/// - SCL: GPIO 27
/// - Frequency: 100 kHz
pub struct I2cImpl {
    driver: EspI2cDriver<'static>,
}

impl I2cImpl {
    /// Create a new I2C driver.
    /// Takes ownership of the I2C0 peripheral and the SDA/SCL pins.
    pub fn new<T1: Pin + InputPin + OutputPin, T2: Pin + InputPin + OutputPin>(
        i2c0: I2C0,
        sda: T1,
        scl: T2,
    ) -> Result<Self, I2cError> {
        let config = Config::new().baudrate(esp_idf_hal::units::Hertz(100_000));
        let driver = EspI2cDriver::new(i2c0, sda, scl, &config)
            .map_err(|_| I2cError::BusError)?;
        Ok(Self { driver })
    }
}

impl I2cDriver for I2cImpl {
    fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), I2cError> {
        self.driver
            .write(addr, data, esp_idf_hal::delay::BLOCK)
            .map_err(|_| I2cError::BusError)
    }

    fn read(&mut self, addr: u8, len: usize) -> Result<Vec<u8>, I2cError> {
        let mut buffer = vec![0u8; len];
        self.driver
            .read(addr, &mut buffer, esp_idf_hal::delay::BLOCK)
            .map_err(|_| I2cError::BusError)?;
        Ok(buffer)
    }

    fn write_read(
        &mut self,
        addr: u8,
        write: &[u8],
        read_len: usize,
    ) -> Result<Vec<u8>, I2cError> {
        let mut buffer = vec![0u8; read_len];
        self.driver
            .write_read(addr, write, &mut buffer, esp_idf_hal::delay::BLOCK)
            .map_err(|_| I2cError::BusError)?;
        Ok(buffer)
    }
}