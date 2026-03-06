use thiserror::Error;

#[derive(Debug, Error)]
pub enum GpioError {
    #[error("Invalid pin {0}")]
    InvalidPin(u8),
    #[error("Pin not configured")]
    NotConfigured,
    #[error("Hardware error")]
    HardwareError,
}

pub trait GpioDriver: Send + Sync {
    fn set_output(&mut self, pin: u8) -> Result<(), GpioError>;
    fn set_input(&mut self, pin: u8) -> Result<(), GpioError>;
    fn write(&mut self, pin: u8, high: bool) -> Result<(), GpioError>;
    fn read(&self, pin: u8) -> Result<bool, GpioError>;
}
