use async_trait::async_trait;
use heizbox_hal::{WifiDriver, WifiError, IpAddr};

/// ESP32 WiFi stub.
/// Replace with `esp_idf_svc::wifi::EspWifi` in production.
pub struct WifiImpl {
    connected: bool,
    ip: Option<IpAddr>,
}

impl WifiImpl {
    pub fn new() -> Self {
        Self {
            connected: false,
            ip: None,
        }
    }
}

impl Default for WifiImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WifiDriver for WifiImpl {
    async fn connect(&mut self, _ssid: &str, _password: &str) -> Result<(), WifiError> {
        self.connected = true;
        self.ip = Some(IpAddr([192, 168, 1, 42]));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), WifiError> {
        self.connected = false;
        self.ip = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_ip(&self) -> Option<IpAddr> {
        self.ip
    }

    fn get_signal_strength(&self) -> i8 {
        -50
    }
}
