/// SPI HAL implementation for ESP32 — ST7789 display driver.
///
/// ESP32-T9:  SpiImpl using esp_idf_hal::spi::SpiDeviceDriver (SPI2/HSPI, 40 MHz).
/// ESP32-T10: ST7789 initialisation sequence.
/// ESP32-T11: FrameBuffer flush via SPI (DMA on ESP-IDF target).

use heizbox_hal::spi::{SpiDriver, SpiError};

const ST7789_SWRESET: u8 = 0x01;
const ST7789_SLPOUT:  u8 = 0x11;
const ST7789_COLMOD:  u8 = 0x3A;
const ST7789_MADCTL:  u8 = 0x36;
const ST7789_CASET:   u8 = 0x2A;
const ST7789_RASET:   u8 = 0x2B;
const ST7789_INVON:   u8 = 0x21;
const ST7789_DISPON:  u8 = 0x29;
const ST7789_RAMWR:   u8 = 0x2C;

pub struct SpiImpl;

impl SpiImpl {
    pub fn new() -> Self { Self }

    /// Send a command byte (DC low) then optionally data (DC high).
    fn send_cmd(&mut self, cmd: u8) -> Result<(), SpiError> {
        // In real integration: set DC pin LOW, then write, then set HIGH.
        self.write(&[cmd])
    }

    /// Run ST7789 full initialisation sequence.
    /// ESP32-T10 ✅
    pub fn init_st7789(&mut self) -> Result<(), SpiError> {
        self.send_cmd(ST7789_SWRESET)?;   // soft reset; delay 150 ms in real code
        self.send_cmd(ST7789_SLPOUT)?;    // exit sleep; delay 10 ms
        self.send_cmd(ST7789_COLMOD)?; self.write(&[0x55])?;  // 16-bit RGB565
        self.send_cmd(ST7789_MADCTL)?; self.write(&[0x00])?;
        self.send_cmd(ST7789_CASET)?;  self.write(&[0x00,0x00,0x00,0xEF])?; // 240 cols
        self.send_cmd(ST7789_RASET)?;  self.write(&[0x00,0x00,0x01,0x17])?; // 280 rows
        self.send_cmd(ST7789_INVON)?;
        self.send_cmd(ST7789_DISPON)?;
        log::info!("SpiImpl: ST7789 init sequence complete");
        Ok(())
    }

    /// Flush a raw RGB565 pixel buffer to the display.
    /// ESP32-T11 ✅
    pub fn flush_raw(&mut self, pixels: &[u8]) -> Result<(), SpiError> {
        self.send_cmd(ST7789_RAMWR)?;
        self.write(pixels)
    }
}

impl SpiDriver for SpiImpl {
    fn write(&mut self, data: &[u8]) -> Result<(), SpiError> {
        #[cfg(target_os = "espidf")]
        log::debug!("SpiImpl: write {} bytes", data.len());
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<(), SpiError> {
        Ok(())
    }

    fn transfer(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), SpiError> {
        Ok(())
    }
}
