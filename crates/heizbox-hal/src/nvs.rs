/// HAL-T2 ✅: NvsDriver uses Option<T> return types for missing keys instead
/// of Result<T, NvsError::KeyNotFound>, which makes absence explicit and
/// reduces boilerplate at call sites.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NvsError {
    /// The requested key does not exist in the given namespace.
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Type mismatch for key")]
    TypeMismatch,
    #[error("NVS storage full")]
    NvsFull,
    #[error("Invalid value")]
    InvalidValue,
    #[error("NVS uninitialised")]
    Uninitialized,
    /// Wraps low-level I/O errors (e.g. flash write failure).
    #[error("NVS I/O error: {0}")]
    Io(String),
}

/// HAL-T2 ✅: All getter methods return `Option<T>` — `None` for missing keys.
pub trait NvsDriver {
    fn get_u8(&mut self, ns: &str, key: &str) -> Result<Option<u8>, NvsError>;
    fn set_u8(&mut self, ns: &str, key: &str, v: u8) -> Result<(), NvsError>;

    fn get_u16(&mut self, ns: &str, key: &str) -> Result<Option<u16>, NvsError>;
    fn set_u16(&mut self, ns: &str, key: &str, v: u16) -> Result<(), NvsError>;

    fn get_u32(&mut self, ns: &str, key: &str) -> Result<Option<u32>, NvsError>;
    fn set_u32(&mut self, ns: &str, key: &str, v: u32) -> Result<(), NvsError>;

    fn get_blob(&mut self, ns: &str, key: &str) -> Result<Option<heapless::Vec<u8, 64>>, NvsError>;
    fn set_blob(&mut self, ns: &str, key: &str, data: &[u8]) -> Result<(), NvsError>;

    fn erase_ns(&mut self, ns: &str) -> Result<(), NvsError>;
}
