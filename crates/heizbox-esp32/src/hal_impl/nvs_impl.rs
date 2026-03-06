use heizbox_hal::{NvsDriver, NvsError};
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsCustom};
use esp_idf_svc::handle::RawHandle;
use std::sync::Mutex;
use esp_idf_svc::sys::{
    nvs_commit,
    nvs_erase_all,
    ESP_ERR_NVS_TYPE_MISMATCH,
    ESP_ERR_NVS_INVALID_HANDLE,
    ESP_ERR_NVS_NOT_INITIALIZED,
    ESP_ERR_NVS_PART_NOT_FOUND,
    ESP_ERR_NVS_NO_FREE_PAGES,
    ESP_ERR_NVS_NOT_ENOUGH_SPACE,
    ESP_ERR_NVS_INVALID_NAME,
    ESP_ERR_NVS_INVALID_LENGTH,
    ESP_ERR_NVS_VALUE_TOO_LONG,
    ESP_ERR_NVS_INVALID_STATE,
    ESP_ERR_NVS_READ_ONLY,
};

/// Real NVS implementation using esp-idf-svc.
pub struct NvsImpl {
    partition: Mutex<EspNvsPartition<NvsCustom>>,
}

impl NvsImpl {
    pub fn new() -> Result<Self, NvsError> {
        let partition = EspNvsPartition::<NvsCustom>::take("nvs")
            .map_err(|_| NvsError::Uninitialized)?;
        Ok(Self {
            partition: Mutex::new(partition),
        })
    }

    fn open_namespace(&self, namespace: &str, read_write: bool) -> Result<EspNvs<NvsCustom>, NvsError> {
        let partition_guard = self.partition.lock().map_err(|_| NvsError::Uninitialized)?;
        let partition = partition_guard.clone();
        drop(partition_guard);
        EspNvs::new(partition, namespace, read_write).map_err(|e| map_esp_error_code(e.code()))
    }
}

fn map_esp_error_code(code: i32) -> NvsError {
    match code {
        ESP_ERR_NVS_TYPE_MISMATCH => NvsError::TypeMismatch,
        ESP_ERR_NVS_INVALID_HANDLE
        | ESP_ERR_NVS_NOT_INITIALIZED
        | ESP_ERR_NVS_PART_NOT_FOUND => NvsError::Uninitialized,
        ESP_ERR_NVS_NO_FREE_PAGES | ESP_ERR_NVS_NOT_ENOUGH_SPACE => NvsError::NvsFull,
        ESP_ERR_NVS_INVALID_NAME
        | ESP_ERR_NVS_INVALID_LENGTH
        | ESP_ERR_NVS_VALUE_TOO_LONG
        | ESP_ERR_NVS_INVALID_STATE
        | ESP_ERR_NVS_READ_ONLY => NvsError::InvalidValue,
        _ => NvsError::Uninitialized,
    }
}

impl NvsDriver for NvsImpl {
    fn get_u8(&self, namespace: &str, key: &str, default: u8) -> Result<u8, NvsError> {
        let nvs = self.open_namespace(namespace, false)?;
        match nvs.get_u8(key) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Ok(default),
            Err(e) => Err(map_esp_error_code(e.code())),
        }
    }

    fn get_u16(&self, namespace: &str, key: &str, default: u16) -> Result<u16, NvsError> {
        let nvs = self.open_namespace(namespace, false)?;
        match nvs.get_u16(key) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Ok(default),
            Err(e) => Err(map_esp_error_code(e.code())),
        }
    }

    fn get_u32(&self, namespace: &str, key: &str, default: u32) -> Result<u32, NvsError> {
        let nvs = self.open_namespace(namespace, false)?;
        match nvs.get_u32(key) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Ok(default),
            Err(e) => Err(map_esp_error_code(e.code())),
        }
    }

    fn get_i32(&self, namespace: &str, key: &str, default: i32) -> Result<i32, NvsError> {
        let nvs = self.open_namespace(namespace, false)?;
        match nvs.get_i32(key) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Ok(default),
            Err(e) => Err(map_esp_error_code(e.code())),
        }
    }

    fn get_f32(&self, namespace: &str, key: &str, default: f32) -> Result<f32, NvsError> {
        let nvs = self.open_namespace(namespace, false)?;
        match nvs.get_u32(key) {
            Ok(Some(bits)) => Ok(f32::from_bits(bits)),
            Ok(None) => Ok(default),
            Err(e) => Err(map_esp_error_code(e.code())),
        }
    }

    fn get_str(&self, namespace: &str, key: &str) -> Result<String, NvsError> {
        let nvs = self.open_namespace(namespace, false)?;
        let len_opt = nvs.str_len(key).map_err(|e| map_esp_error_code(e.code()))?;
        match len_opt {
            Some(len) => {
                if len == 0 {
                    return Ok(String::new());
                }
                let mut buf = vec![0u8; len];
                match nvs.get_str(key, &mut buf) {
                    Ok(Some(s)) => Ok(s.to_string()),
                    Ok(None) => Ok(String::new()),
                   Err(e) => Err(map_esp_error_code(e.code())),
                }
            }
            None => Ok(String::new()),
        }
    }

    fn set_u8(&self, namespace: &str, key: &str, v: u8) -> Result<(), NvsError> {
        let mut nvs = self.open_namespace(namespace, true)?;
        nvs.set_u8(key, v).map_err(|e| map_esp_error_code(e.code()))
    }

    fn set_u16(&self, namespace: &str, key: &str, v: u16) -> Result<(), NvsError> {
        let mut nvs = self.open_namespace(namespace, true)?;
        nvs.set_u16(key, v).map_err(|e| map_esp_error_code(e.code()))
    }

    fn set_u32(&self, namespace: &str, key: &str, v: u32) -> Result<(), NvsError> {
        let mut nvs = self.open_namespace(namespace, true)?;
        nvs.set_u32(key, v).map_err(|e| map_esp_error_code(e.code()))
    }

    fn set_i32(&self, namespace: &str, key: &str, v: i32) -> Result<(), NvsError> {
        let mut nvs = self.open_namespace(namespace, true)?;
        nvs.set_i32(key, v).map_err(|e| map_esp_error_code(e.code()))
    }

    fn set_f32(&self, namespace: &str, key: &str, v: f32) -> Result<(), NvsError> {
        let mut nvs = self.open_namespace(namespace, true)?;
        nvs.set_u32(key, v.to_bits()).map_err(|e| map_esp_error_code(e.code()))
    }

    fn set_str(&self, namespace: &str, key: &str, v: &str) -> Result<(), NvsError> {
        let mut nvs = self.open_namespace(namespace, true)?;
        nvs.set_str(key, v).map_err(|e| map_esp_error_code(e.code()))
    }

    fn erase(&self, namespace: &str) -> Result<(), NvsError> {
        let nvs = self.open_namespace(namespace, true)?;
        // Retrieve the nvs_handle_t as u32 (nvs_handle_t is u32 in bindings).
        let handle_u32 = nvs.handle() as u32;
        unsafe {
            let err = nvs_erase_all(handle_u32);
            if err != 0 {
                return Err(map_esp_error_code(err));
            }
            let err = nvs_commit(handle_u32);
            if err != 0 {
                return Err(map_esp_error_code(err));
            }
        }
        Ok(())
    }
}
