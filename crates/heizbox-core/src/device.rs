use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device not ready")]
    NotReady,
    #[error("Device error: {0}")]
    Other(String),
}

<<<<<<< ours
/// Core device capability trait.
#[async_trait]
pub trait Device {
    /// Returns `Ok(())` when the device is operational.
=======
#[async_trait]
pub trait Device {
>>>>>>> theirs
    async fn status(&self) -> Result<(), DeviceError>;
}
