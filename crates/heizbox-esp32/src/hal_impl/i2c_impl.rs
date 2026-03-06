use esp_idf_hal::i2c::{I2cDriver as EspI2cDriver, config::Config};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::units::Hertz;
use heizbox_hal::{I2cDriver, I2cError};

/// ESP32 I2C driver using `esp-idf-hal`.
///
/// Pin configuration (from hardware spec):
/// - SCL: GPIO 27
/// - SDA: GPIO 26
/// - Frequency: 100 kHz
pub struct I2cImpl {
    driver: EspI2cDriver<'static>,
}

impl I2cImpl {
    pub fn new() -> Result<Self, I2cError> {
        let peripherals = Peripherals::take().map_err(|_| I2cError::BusError)?;

        // Configure I2C pins: SDA=26, SCL=27, 100 kHz
        let sda = peripherals.pins.gpio26;
        let scl = peripherals.pins.gpio27;

        let config = Config::new().baudrate(Hertz(100_000));
        let driver = EspI2cDriver::new(peripherals.i2c0, sda, scl, &config)
            .map_err(|_| I2cError::BusError)?;

        Ok(Self { driver })
    }
}

impl Default for I2cImpl {
    fn default() -> Self {
        Self::new().expect("I2C initialization failed")
    }
}

impl I2cDriver for I2cImpl {
    fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), I2cError> {
        self.driver
            .write(addr, data, BLOCK)
            .map_err(|_| I2cError::BusError)
    }

    fn read(&mut self, addr: u8, len: usize) -> Result<Vec<u8>, I2cError> {
        let mut buffer = vec![0u8; len];
        self.driver
            .read(addr, &mut buffer, BLOCK)
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
            .write_read(addr, write, &mut buffer, BLOCK)
            .map_err(|_| I2cError::BusError)?;
        Ok(buffer)
    }
}
