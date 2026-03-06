use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpiError {
    #[error("SPI bus error")]
    BusError,
    #[error("Chip-select error")]
    ChipSelectError,
    #[error("Timeout")]
    Timeout,
}

pub trait SpiDriver: Send + Sync {
    fn write(&mut self, data: &[u8]) -> Result<(), SpiError>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), SpiError>;
    fn transfer(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), SpiError>;
}
