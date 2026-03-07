use heizbox_hal::{WifiDriver, WifiError, IpAddr};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::Configuration;
use esp_idf_svc::wifi::ClientConfiguration;
use std::time::{Instant, Duration};
use std::thread;
use std::sync::Mutex;
use heapless;

/// ESP32 WiFi implementation using esp-idf-svc.
/// Wraps EspWifi in Mutex to satisfy `Send + Sync` bounds of WifiDriver.
pub struct WifiImpl {
    wifi: Mutex<EspWifi<'static>>,
    connected: bool,
    ip: Option<IpAddr>,
    connected_ssid: Option<heapless::String<32>>,
}

impl WifiImpl {
    pub fn new(
        modem: Modem,
        sysloop: EspSystemEventLoop,
    ) -> Result<Self, WifiError> {
        let wifi = EspWifi::new(modem, sysloop, None)
            .map_err(|_| WifiError::NotSupported)?;
        Ok(Self {
            wifi: Mutex::new(wifi),
            connected: false,
            ip: None,
            connected_ssid: None,
        })
    }
}

impl WifiDriver for WifiImpl {
    fn connect(&mut self, ssid: &str, password: &str) -> Result<(), WifiError> {
        // Convert to heapless::String with proper capacity
        let ssid_h = heapless::String::<32>::try_from(ssid)
            .map_err(|_| WifiError::NotSupported)?;
        let password_h = heapless::String::<64>::try_from(password)
            .map_err(|_| WifiError::NotSupported)?;
        // Clone ssid before moving into config
        let ssid_for_self = ssid_h.clone();

        let config = Configuration::Client(ClientConfiguration {
            ssid: ssid_h,
            password: password_h,
            ..Default::default()
        });

        let mut wifi = self.wifi.lock().unwrap();
        wifi.set_configuration(&config)
            .map_err(|_| WifiError::NotSupported)?;
        wifi.start().map_err(|_| WifiError::NotSupported)?;
        wifi.connect().map_err(|_| WifiError::NotSupported)?;

        // Wait until connected
        let start = Instant::now();
        let timeout = Duration::from_secs(10);
        loop {
            match wifi.is_connected() {
                Ok(true) => break,
                _ => {}
            }
            if start.elapsed() >= timeout {
                return Err(WifiError::ConnectionTimeout);
            }
            thread::sleep(Duration::from_millis(100));
        }

        self.connected = true;
        self.connected_ssid = Some(ssid_for_self);
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), WifiError> {
        let mut wifi = self.wifi.lock().unwrap();
        wifi.disconnect().map_err(|_| WifiError::NotSupported)?;
        self.connected = false;
        self.ip = None;
        self.connected_ssid = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_ip(&self) -> Option<IpAddr> {
        self.ip
    }

    fn get_signal_strength(&self) -> i8 {
        if self.connected {
            let wifi = self.wifi.lock().unwrap();
            match wifi.get_rssi() {
                Ok(rssi) => rssi as i8,
                Err(_) => -50,
            }
        } else {
            -50
        }
    }

    fn ssid(&self) -> Option<&str> {
        self.connected_ssid.as_deref()
    }
}
