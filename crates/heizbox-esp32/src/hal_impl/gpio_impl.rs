use esp_idf_svc::sys::*;
use heizbox_hal::{GpioDriver, GpioError};

/// ESP32 GPIO driver using direct esp-idf-svc sys calls.
///
/// Pins are configured at initialization according to the hardware specification.
/// Only actual ESP32 GPIO pins are configured here; joystick inputs are handled
/// via the PCF8574 I2C expander and are not part of this driver.
///
/// Configured pins:
/// - MOSFET gate (GPIO32) → Output
/// - Fire button (GPIO13) → Input with Pull-Up
/// - Display reset (GPIO15) → Output
/// - Display backlight (GPIO16) → Output
///
/// Note: Display DC (GPIO4) and all joystick pins (GPIO0,1,2,3,4) are intentionally
/// not configured as they either conflict with I2C-expander joystick signals or
/// are reserved for other peripherals.
pub struct GpioImpl;

impl GpioImpl {
    /// Construct a new GPIO driver and pre-configure required pins.
    pub fn new() -> Result<Self, GpioError> {
        // Helper to set direction and check error (safe wrapper)
        fn set_dir(pin: u8, mode: u32) -> Result<(), GpioError> {
            unsafe {
                let ret = gpio_set_direction(pin as i32, mode);
                if ret == ESP_OK as i32 {
                    Ok(())
                } else {
                    Err(GpioError::HardwareError)
                }
            }
        }

        unsafe {
            // Configure MOSFET gate (GPIO32) as output
            set_dir(32, GPIO_MODE_DEF_OUTPUT)?;
            // Configure Fire button (GPIO13) as input with pull-up
            set_dir(13, GPIO_MODE_DEF_INPUT)?;
            let ret = gpio_set_pull_mode(13 as i32, gpio_pull_mode_t_GPIO_PULLUP_ONLY);
            if ret != ESP_OK as i32 {
                return Err(GpioError::HardwareError);
            }
            // Display pins as outputs: RST (GPIO15), BL (GPIO16)
            for &pin in &[15, 16] {
                set_dir(pin, GPIO_MODE_DEF_OUTPUT)?;
            }
        }
        Ok(Self)
    }
}

impl Default for GpioImpl {
    fn default() -> Self {
        Self::new().expect("GPIO initialization failed")
    }
}

impl GpioDriver for GpioImpl {
    fn set_output(&mut self, pin: u8) -> Result<(), GpioError> {
        unsafe {
            let ret = gpio_set_direction(pin as i32, GPIO_MODE_DEF_OUTPUT);
            if ret == ESP_OK as i32 {
                Ok(())
            } else {
                Err(GpioError::InvalidPin(pin))
            }
        }
    }

    fn set_input(&mut self, pin: u8) -> Result<(), GpioError> {
        unsafe {
            let ret = gpio_set_direction(pin as i32, GPIO_MODE_DEF_INPUT);
            if ret != ESP_OK as i32 {
                return Err(GpioError::InvalidPin(pin));
            }
            // Also enable pull-up by default as per spec
            let ret2 = gpio_set_pull_mode(pin as i32, gpio_pull_mode_t_GPIO_PULLUP_ONLY);
            if ret2 != ESP_OK as i32 {
                Err(GpioError::HardwareError)
            } else {
                Ok(())
            }
        }
    }

    fn write(&mut self, pin: u8, high: bool) -> Result<(), GpioError> {
        unsafe {
            let level = if high { 1 } else { 0 };
            let ret = gpio_set_level(pin as i32, level);
            if ret == ESP_OK as i32 {
                Ok(())
            } else {
                Err(GpioError::NotConfigured)
            }
        }
    }

    fn read(&self, pin: u8) -> Result<bool, GpioError> {
        unsafe {
            let level = gpio_get_level(pin as i32);
            Ok(level != 0)
        }
    }
}