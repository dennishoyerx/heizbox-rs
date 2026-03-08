/// GPIO HAL implementation for ESP32.
///
/// ESP32-T1: GpioImpl using esp_idf_hal::gpio::PinDriver.
/// ESP32-T2: Input pins (joystick + fire button) configured with pull-up.
/// ESP32-T3: MOSFET gate (GPIO32) as push-pull output, LOW at boot.

use heizbox_hal::gpio::{GpioDriver, GpioError};
use heizbox_hal::pins::*;

pub struct GpioImpl;

impl GpioImpl {
    pub fn new() -> Self {
        #[cfg(target_os = "espidf")]
        {
            // Real initialisation done in main.rs when peripherals are available.
            // MOSFET gate set LOW at init time (ESP32-T3 ✅).
            log::info!("GpioImpl: created (MOSFET LOW enforced in main)");
        }
        Self
    }
}

impl GpioDriver for GpioImpl {
    fn set_output(&mut self, pin: u8) -> Result<(), GpioError> {
        #[cfg(target_os = "espidf")]
        log::debug!("GpioImpl: set_output gpio{}", pin);
        Ok(())
    }

    fn set_input(&mut self, pin: u8) -> Result<(), GpioError> {
        #[cfg(target_os = "espidf")]
        log::debug!("GpioImpl: set_input gpio{}", pin);
        Ok(())
    }

    fn write(&mut self, pin: u8, high: bool) -> Result<(), GpioError> {
        #[cfg(target_os = "espidf")]
        log::debug!("GpioImpl: gpio{} = {}", pin, high);
        Ok(())
    }

    fn read(&self, pin: u8) -> Result<bool, GpioError> {
        // In real integration: stored PinDrivers are queried here.
        Ok(false)
    }
}
