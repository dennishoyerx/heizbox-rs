use heizbox_hal::wifi::{WifiDriver, WifiError, IpAddr};
use heizbox_core::network::ExponentialBackoff;

pub struct WifiImpl {
    backoff: ExponentialBackoff,
    connected: bool,
}

impl WifiImpl {
    pub fn new() -> Self {
        Self { backoff: ExponentialBackoff::new(2_000, 60_000), connected: false }
    }

    /// Handle disconnect event with backoff delay.
    /// ESP32-T16 ✅
    pub fn handle_disconnect(&mut self) -> u32 {
        self.connected = false;
        let delay = self.backoff.next_delay_ms();
        log::warn!("WifiImpl: disconnected, retry in {} ms", delay);
        delay
    }
}

impl WifiDriver for WifiImpl {
    fn connect(&mut self, ssid: &str, password: &str) -> Result<(), WifiError> {
        #[cfg(target_os = "espidf")]
        log::info!("WifiImpl: connecting to '{}'", ssid);
        self.connected = true;
        self.backoff.reset();
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), WifiError> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool { self.connected }

    fn get_ip(&self) -> Option<IpAddr> {
        if self.connected { Some(IpAddr([192,168,1,100])) } else { None }
    }

    fn get_signal_strength(&self) -> i8 { -60 }

    fn ssid(&self) -> Option<&str> {
        if self.connected { Some("heizbox-wifi") } else { None }
    }
}
