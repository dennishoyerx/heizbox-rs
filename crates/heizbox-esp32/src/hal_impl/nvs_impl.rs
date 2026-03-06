use heizbox_hal::{NvsDriver, NvsError};

/// In-memory NVS stub (all reads return defaults, writes are no-ops).
/// Replace with `esp_idf_svc::nvs::EspNvs` in production.
pub struct NvsImpl;

impl NvsImpl {
    pub fn new() -> Result<Self, NvsError> {
        Ok(Self)
    }
}

impl Default for NvsImpl {
    fn default() -> Self {
        Self
    }
}

impl NvsDriver for NvsImpl {
    fn get_u8(&self, _ns: &str, _key: &str, default: u8) -> Result<u8, NvsError> { Ok(default) }
    fn get_u16(&self, _ns: &str, _key: &str, default: u16) -> Result<u16, NvsError> { Ok(default) }
    fn get_u32(&self, _ns: &str, _key: &str, default: u32) -> Result<u32, NvsError> { Ok(default) }
    fn get_i32(&self, _ns: &str, _key: &str, default: i32) -> Result<i32, NvsError> { Ok(default) }
    fn get_f32(&self, _ns: &str, _key: &str, default: f32) -> Result<f32, NvsError> { Ok(default) }
    fn get_str(&self, _ns: &str, _key: &str) -> Result<String, NvsError> { Ok(String::new()) }

    fn set_u8(&self, _ns: &str, _key: &str, _v: u8) -> Result<(), NvsError> { Ok(()) }
    fn set_u16(&self, _ns: &str, _key: &str, _v: u16) -> Result<(), NvsError> { Ok(()) }
    fn set_u32(&self, _ns: &str, _key: &str, _v: u32) -> Result<(), NvsError> { Ok(()) }
    fn set_i32(&self, _ns: &str, _key: &str, _v: i32) -> Result<(), NvsError> { Ok(()) }
    fn set_f32(&self, _ns: &str, _key: &str, _v: f32) -> Result<(), NvsError> { Ok(()) }
    fn set_str(&self, _ns: &str, _key: &str, _v: &str) -> Result<(), NvsError> { Ok(()) }

    fn erase(&self, _ns: &str) -> Result<(), NvsError> { Ok(()) }
}
