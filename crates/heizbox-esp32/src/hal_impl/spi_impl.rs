use esp_idf_hal::spi::{
    SpiDriver, SpiDeviceDriver, config::{DriverConfig, Config, Mode, MODE_0}, Dma,
};
use esp_idf_hal::gpio::{Gpio14, Gpio12, Gpio8, Gpio5};
use esp_idf_hal::spi::SPI2;
use esp_idf_hal::units::Hertz;
use heizbox_hal::{SpiDriver as HeizboxSpiDriver, SpiError};

/// ESP32 SPI driver using `esp_idf_hal` on SPI2 (HSPI) with manual CS.
///
/// Pin configuration (from pins.rs):
/// - SCK: GPIO14
/// - MOSI: GPIO12
/// - MISO: not used
/// - CS: handled separately via GPIO
///
/// Clock: 40 MHz.
pub struct SpiImpl {
    driver: SpiDeviceDriver<'static, &'static SpiDriver<'static>>,
}

impl SpiImpl {
    /// Create a new SPI driver on SPI2 with the given pins.
    ///
    /// # Arguments
    /// - `spi`: the SPI2 peripheral (take from `Peripherals::take().spi2`)
    /// - `sclk`: GPIO pin for clock (GPIO14)
    /// - `mosi`: GPIO pin for MOSI (GPIO12)
    ///
    /// Returns an error if the driver cannot be initialized.
    pub fn new(
        spi: SPI2,
        sclk: Gpio14,
        mosi: Gpio12,
    ) -> Result<Self, SpiError> {
        // First, create the low-level SPI bus driver.
        // Enable DMA for high-throughput display flushing (target ~20 fps).
        let bus_config = DriverConfig::new().dma(Dma::Auto(4096));
        let bus = SpiDriver::new(spi, sclk, mosi, None::<Gpio8>, &bus_config)
            .map_err(|_| SpiError::BusError)?;
        // Leak the bus to get a 'static reference (no destructor needed for whole program)
        let bus_ref: &'static SpiDriver = Box::leak(Box::new(bus));

        // Then create the SPI device (single device) with no hardware CS (we handle manually)
        let device_config = Config::new()
            .baudrate(Hertz(40_000_000))
            .data_mode(MODE_0)
            .write_only(true); // Display write-only

        let driver = SpiDeviceDriver::new(bus_ref, None::<Gpio5>, &device_config)
            .map_err(|_| SpiError::BusError)?;

        Ok(Self { driver })
    }
}

impl Default for SpiImpl {
    fn default() -> Self {
        panic!("SpiImpl::default() is not valid; use SpiImpl::new(spi2, sclk, mosi)")
    }
}

impl HeizboxSpiDriver for SpiImpl {
    fn write(&mut self, data: &[u8]) -> Result<(), SpiError> {
        self.driver
            .write(data)
            .map_err(|_| SpiError::BusError)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<(), SpiError> {
        self.driver
            .read(buffer)
            .map_err(|_| SpiError::BusError)
    }

    fn transfer(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), SpiError> {
        // Note: esp-idf transfer takes (read, write) order.
        self.driver
            .transfer(read, write)
            .map_err(|_| SpiError::BusError)
    }
}
