use thiserror::Error;

#[derive(Debug, Error)]
pub enum NvsError {
    #[error("Key not found")]
    KeyNotFound,
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("NVS storage full")]
    NvsFull,
    #[error("Invalid value")]
    InvalidValue,
    #[error("NVS uninitialised")]
    Uninitialized,
}

pub trait NvsDriver: Send + Sync {
    fn get_u8(&self, namespace: &str, key: &str, default: u8) -> Result<u8, NvsError>;
    fn get_u16(&self, namespace: &str, key: &str, default: u16) -> Result<u16, NvsError>;
    fn get_u32(&self, namespace: &str, key: &str, default: u32) -> Result<u32, NvsError>;
    fn get_i32(&self, namespace: &str, key: &str, default: i32) -> Result<i32, NvsError>;
    fn get_f32(&self, namespace: &str, key: &str, default: f32) -> Result<f32, NvsError>;
    fn get_str(&self, namespace: &str, key: &str) -> Result<String, NvsError>;

    fn set_u8(&self, namespace: &str, key: &str, value: u8) -> Result<(), NvsError>;
    fn set_u16(&self, namespace: &str, key: &str, value: u16) -> Result<(), NvsError>;
    fn set_u32(&self, namespace: &str, key: &str, value: u32) -> Result<(), NvsError>;
    fn set_i32(&self, namespace: &str, key: &str, value: i32) -> Result<(), NvsError>;
    fn set_f32(&self, namespace: &str, key: &str, value: f32) -> Result<(), NvsError>;
    fn set_str(&self, namespace: &str, key: &str, value: &str) -> Result<(), NvsError>;

    fn erase(&self, namespace: &str) -> Result<(), NvsError>;
}
