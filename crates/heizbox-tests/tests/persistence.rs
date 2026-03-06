#[cfg(test)]
mod persistence_tests {
    use heizbox_core::heater::HeaterConfig;

    /// Simple in-memory NVS backed by a HashMap for host-side tests.
    mod mock_nvs {
        use std::collections::HashMap;
        use std::sync::Mutex;
        use heizbox_core::error::NvsError as CoreNvsError;

        pub struct MockNvs {
            store: Mutex<HashMap<String, String>>,
        }

        impl MockNvs {
            pub fn new() -> Self {
                Self {
                    store: Mutex::new(HashMap::new()),
                }
            }

            pub fn set(&self, ns: &str, key: &str, value: &str) {
                self.store.lock().unwrap()
                    .insert(format!("{ns}/{key}"), value.to_string());
            }

            pub fn get(&self, ns: &str, key: &str) -> Option<String> {
                self.store.lock().unwrap()
                    .get(&format!("{ns}/{key}"))
                    .cloned()
            }
        }
    }

    /// Verify HeaterConfig defaults are returned when the store is empty.
    #[test]
    fn heater_config_defaults() {
        let config = HeaterConfig::with_defaults();
        assert_eq!(config.power, 100);
        assert_eq!(config.target_temp, 200);
        assert_eq!(config.auto_stop_time_ms, 90_000);
    }
}
