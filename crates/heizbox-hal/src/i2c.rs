use thiserror::Error;

#[derive(Debug, Error)]
pub enum I2cError {
    #[error("I2C bus error")]
    BusError,
    #[error("Address NAK for 0x{0:02x}")]
    AddressNak(u8),
    #[error("Data NAK")]
    DataNak,
    #[error("Timeout")]
    Timeout,
}

pub trait I2cDriver: Send + Sync {
    fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), I2cError>;
    fn read(&mut self, addr: u8, len: usize) -> Result<Vec<u8>, I2cError>;
    fn write_read(
        &mut self,
        addr: u8,
        write: &[u8],
        read_len: usize,
    ) -> Result<Vec<u8>, I2cError>;
}
