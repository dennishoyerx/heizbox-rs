use heizbox_hal::{GpioDriver, SpiDriver};
use heizbox_app::screen::FrameBuffer;
use esp_idf_hal::delay::FreeRtos;
use heizbox_hal::SpiError;
use heizbox_hal::pins::display::{DC, RST, BL, CS};

/// Manages the ST7789 display over SPI.
pub struct DisplayManager {
    spi: Box<dyn SpiDriver + Send>,
    gpio: Box<dyn GpioDriver + Send>,
    width: u16,
    height: u16,
}

impl DisplayManager {
    /// Create a new DisplayManager with the given SPI driver and GPIO driver.
    ///
    /// The SPI driver should be already initialized for 40 MHz.
    pub fn new(
        spi: Box<dyn SpiDriver + Send>,
        gpio: Box<dyn GpioDriver + Send>,
        width: u16,
        height: u16,
    ) -> Self {
        Self { spi, gpio, width, height }
    }

    /// Initialize the display: configure pins, hardware reset and send init commands.
    pub fn init(&mut self) -> Result<(), SpiError> {
        // Configure display control pins as outputs
        self.gpio.set_output(RST).map_err(|_| SpiError::BusError)?;
        self.gpio.set_output(DC).map_err(|_| SpiError::BusError)?;
        self.gpio.set_output(BL).map_err(|_| SpiError::BusError)?;
        self.gpio.set_output(CS).map_err(|_| SpiError::BusError)?;

        // Hardware reset sequence using RST pin (active low)
        self.gpio.write(RST, false).map_err(|_| SpiError::BusError)?; // assert reset
        FreeRtos::delay_ms(10);
        self.gpio.write(RST, true).map_err(|_| SpiError::BusError)?; // deassert
        FreeRtos::delay_ms(120); // wait for power-up

        // Send initialization commands
        self.send_command(0x01)?; // SWRESET - Software reset
        FreeRtos::delay_ms(5);
        self.send_command(0x11)?; // SLPOUT - Sleep out
        FreeRtos::delay_ms(120);

        // Memory data access control: MADCTL
        // For 240x280 portrait with RGB order. Rotation can be adjusted later.
        let madctl = 0x08; // RGB, top-left origin (no mirror)
        self.send_command_with_params(0x36, &[madctl])?;

        // Column address set: CASET
        let w = self.width - 1;
        self.send_command_with_params(0x2A, &[0x00, 0x00, (w >> 8) as u8, w as u8])?;

        // Row address set: RASET
        let h = self.height - 1;
        self.send_command_with_params(0x2B, &[0x00, 0x00, (h >> 8) as u8, h as u8])?;

        // Pixel format: COLMOD (16-bit RGB565)
        self.send_command_with_params(0x3A, &[0x55])?; // 0x55 = 16-bit

        // Display inversion: INVON (0x21)
        self.send_command(0x21)?;

        // Display ON: DISPON (0x29)
        self.send_command(0x29)?;

        // Set backlight on (assuming active high)
        self.gpio.write(BL, true).map_err(|_| SpiError::BusError)?;

        Ok(())
    }

    /// Send a command byte.
    fn send_command(&mut self, cmd: u8) -> Result<(), SpiError> {
        self.gpio.write(CS, false).map_err(|_| SpiError::BusError)?;
        self.gpio.write(DC, false).map_err(|_| SpiError::BusError)?; // command
        let result = self.spi.write(&[cmd]);
        self.gpio.write(CS, true).map_err(|_| SpiError::BusError)?;
        result
    }

    /// Send a command followed by parameters.
    fn send_command_with_params(&mut self, cmd: u8, params: &[u8]) -> Result<(), SpiError> {
        self.gpio.write(CS, false).map_err(|_| SpiError::BusError)?;
        // Command
        self.gpio.write(DC, false).map_err(|_| SpiError::BusError)?;
        let _ = self.spi.write(&[cmd]);
        // Data
        if !params.is_empty() {
            self.gpio.write(DC, true).map_err(|_| SpiError::BusError)?;
            let _ = self.spi.write(params)?;
        }
        self.gpio.write(CS, true).map_err(|_| SpiError::BusError)?;
        Ok(())
    }

    /// Flush a framebuffer to the display.
    ///
    /// The framebuffer must have dimensions matching the display.
    /// This function sets the column and row windows to full screen and then
    /// writes the pixel data.
    pub fn flush(&mut self, fb: &FrameBuffer) -> Result<(), SpiError> {
        // Ensure dimensions match
        if fb.width != self.width || fb.height != self.height {
            return Err(SpiError::BusError);
        }

        // Set column and row address to full window
        let w = self.width - 1;
        self.send_command_with_params(0x2A, &[0x00, 0x00, (w >> 8) as u8, w as u8])?;
        let h = self.height - 1;
        self.send_command_with_params(0x2B, &[0x00, 0x00, (h >> 8) as u8, h as u8])?;

        // Write memory start: RAMWR (0x2C)
        self.send_command(0x2C)?;

        // Send pixel data with CS low and DC high.
        self.gpio.write(CS, false).map_err(|_| SpiError::BusError)?;
        self.gpio.write(DC, true).map_err(|_| SpiError::BusError)?; // data mode
        let result = self.spi.write(&fb.data);
        self.gpio.write(CS, true).map_err(|_| SpiError::BusError)?;
        result
    }
}
