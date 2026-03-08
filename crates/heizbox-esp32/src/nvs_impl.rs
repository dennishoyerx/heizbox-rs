/// NVS implementation using esp_idf_svc::nvs::EspNvs.
///
/// ESP32-T13: All get_*/set_* methods backed by real ESP-IDF NVS flash.
/// ESP32-T14: NVS partition size documented (see partitions.csv).

use heizbox_hal::nvs::{NvsDriver, NvsError};

pub struct NvsImpl;

impl NvsImpl {
    pub fn new() -> Self { Self }
}

impl Default for NvsImpl {
    fn default() -> Self { Self::new() }
}

impl NvsDriver for NvsImpl {
    fn get_u8(&mut self, namespace: &str, key: &str) -> Result<Option<u8>, NvsError> {
        #[cfg(target_os = "espidf")]
        {
            use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
            let nvs_part = EspDefaultNvsPartition::take()
                .map_err(|e| NvsError::Io(e.to_string()))?;
            let nvs = EspNvs::new(nvs_part, namespace, true)
                .map_err(|e| NvsError::Io(e.to_string()))?;
            match nvs.get_u8(key) {
                Ok(Some(v)) => Ok(Some(v)),
                Ok(None)    => Err(NvsError::KeyNotFound(key.into())),
                Err(e)      => Err(NvsError::Io(e.to_string())),
            }
        }
        #[cfg(not(target_os = "espidf"))]
        Err(NvsError::KeyNotFound(key.into()))
    }

    fn set_u8(&mut self, namespace: &str, key: &str, value: u8) -> Result<(), NvsError> {
        #[cfg(target_os = "espidf")]
        {
            use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
            let nvs_part = EspDefaultNvsPartition::take()
                .map_err(|e| NvsError::Io(e.to_string()))?;
            let mut nvs = EspNvs::new(nvs_part, namespace, true)
                .map_err(|e| NvsError::Io(e.to_string()))?;
            nvs.set_u8(key, value)
                .map_err(|e| NvsError::Io(e.to_string()))
        }
        #[cfg(not(target_os = "espidf"))]
        Ok(())
    }

    fn get_u16(&mut self, namespace: &str, key: &str) -> Result<Option<u16>, NvsError> {
        // Store as two u8 keys (hi, lo) — u16 not natively in esp-idf-svc NVS bindings.
        let lo = match self.get_u8(namespace, &format!("{}_lo", key)) {
            Ok(Some(v)) => v,
            _           => return Err(NvsError::KeyNotFound(key.into())),
        };
        let hi = match self.get_u8(namespace, &format!("{}_hi", key)) {
            Ok(Some(v)) => v,
            _           => return Err(NvsError::KeyNotFound(key.into())),
        };
        Ok(Some(u16::from_le_bytes([lo, hi])))
    }

    fn set_u16(&mut self, namespace: &str, key: &str, value: u16) -> Result<(), NvsError> {
        let bytes = value.to_le_bytes();
        self.set_u8(namespace, &format!("{}_lo", key), bytes[0])?;
        self.set_u8(namespace, &format!("{}_hi", key), bytes[1])
    }

    fn get_u32(&mut self, namespace: &str, key: &str) -> Result<Option<u32>, NvsError> {
        #[cfg(target_os = "espidf")]
        {
            use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
            let nvs_part = EspDefaultNvsPartition::take()
                .map_err(|e| NvsError::Io(e.to_string()))?;
            let nvs = EspNvs::new(nvs_part, namespace, true)
                .map_err(|e| NvsError::Io(e.to_string()))?;
            match nvs.get_u32(key) {
                Ok(Some(v)) => Ok(Some(v)),
                Ok(None)    => Err(NvsError::KeyNotFound(key.into())),
                Err(e)      => Err(NvsError::Io(e.to_string())),
            }
        }
        #[cfg(not(target_os = "espidf"))]
        Err(NvsError::KeyNotFound(key.into()))
    }

    fn set_u32(&mut self, namespace: &str, key: &str, value: u32) -> Result<(), NvsError> {
        #[cfg(target_os = "espidf")]
        {
            use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
            let nvs_part = EspDefaultNvsPartition::take()
                .map_err(|e| NvsError::Io(e.to_string()))?;
            let mut nvs = EspNvs::new(nvs_part, namespace, true)
                .map_err(|e| NvsError::Io(e.to_string()))?;
            nvs.set_u32(key, value)
                .map_err(|e| NvsError::Io(e.to_string()))
        }
        #[cfg(not(target_os = "espidf"))]
        Ok(())
    }

    fn get_blob(&mut self, namespace: &str, key: &str) -> Result<Option<heapless::Vec<u8, 64>>, NvsError> {
        #[cfg(target_os = "espidf")]
        {
            use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
            let nvs_part = EspDefaultNvsPartition::take()
                .map_err(|e| NvsError::Io(e.to_string()))?;
            let nvs = EspNvs::new(nvs_part, namespace, true)
                .map_err(|e| NvsError::Io(e.to_string()))?;
            let mut buf = [0u8; 64];
            match nvs.get_raw(key, &mut buf) {
                Ok(Some(n)) => {
                    let mut v = heapless::Vec::new();
                    let _ = v.extend_from_slice(&buf[..n]);
                    Ok(Some(v))
                }
                Ok(None) => Err(NvsError::KeyNotFound(key.into())),
                Err(e)   => Err(NvsError::Io(e.to_string())),
            }
        }
        #[cfg(not(target_os = "espidf"))]
        Err(NvsError::KeyNotFound(key.into()))
    }

    fn set_blob(&mut self, namespace: &str, key: &str, data: &[u8]) -> Result<(), NvsError> {
        #[cfg(target_os = "espidf")]
        {
            use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
            let nvs_part = EspDefaultNvsPartition::take()
                .map_err(|e| NvsError::Io(e.to_string()))?;
            let mut nvs = EspNvs::new(nvs_part, namespace, true)
                .map_err(|e| NvsError::Io(e.to_string()))?;
            nvs.set_raw(key, data)
                .map_err(|e| NvsError::Io(e.to_string()))
        }
        #[cfg(not(target_os = "espidf"))]
        Ok(())
    }
}
