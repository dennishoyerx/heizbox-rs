use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device not ready")]
    NotReady,
    #[error("Device error: {0}")]
    Other(String),
}

#[async_trait]
pub trait Device {
    async fn status(&self) -> Result<(), DeviceError>;
}
