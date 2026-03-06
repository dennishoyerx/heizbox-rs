use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdcError {
    #[error("ADC channel error for pin {0}")]
    ChannelError(u8),
    #[error("ADC conversion error")]
    ConversionError,
    #[error("Not ready")]
    NotReady,
}

pub trait AdcDriver: Send + Sync {
    fn read(&self, pin: u8) -> Result<u16, AdcError>;
}
