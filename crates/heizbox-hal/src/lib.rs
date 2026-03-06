pub mod adc;
pub mod gpio;
pub mod i2c;
pub mod nvs;
pub mod pins;
pub mod spi;
pub mod timer;
pub mod wifi;

// Flat re-exports so callers can write `heizbox_hal::GpioDriver`.
pub use adc::{AdcDriver, AdcError};
pub use gpio::{GpioDriver, GpioError};
pub use i2c::{I2cDriver, I2cError};
pub use nvs::{NvsDriver, NvsError};
pub use spi::{SpiDriver, SpiError};
pub use timer::{TimerDriver, TimerError};
pub use wifi::{WifiDriver, WifiError, IpAddr};
