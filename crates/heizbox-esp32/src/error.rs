//! Maps esp-idf error types to heizbox-hal error types.

use heizbox_hal::{GpioError, I2cError, SpiError, NvsError, WifiError, AdcError, TimerError};

pub fn map_gpio_error(_e: impl core::fmt::Debug) -> GpioError {
    GpioError::HardwareError
}

pub fn map_i2c_error(_e: impl core::fmt::Debug) -> I2cError {
    I2cError::BusError
}

pub fn map_spi_error(_e: impl core::fmt::Debug) -> SpiError {
    SpiError::BusError
}

pub fn map_nvs_error(_e: impl core::fmt::Debug) -> NvsError {
    NvsError::Uninitialized
}

pub fn map_wifi_error(_e: impl core::fmt::Debug) -> WifiError {
    WifiError::AuthFailed
}

pub fn map_adc_error(_e: impl core::fmt::Debug) -> AdcError {
    AdcError::ConversionError
}

pub fn map_timer_error(_e: impl core::fmt::Debug) -> TimerError {
    TimerError::InvalidChannel
}
