use thiserror::Error;

#[derive(Debug, Error)]
pub enum WifiError {
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Connection timeout")]
    ConnectionTimeout,
    #[error("Not supported")]
    NotSupported,
    #[error("Already connected")]
    AlreadyConnected,
}

/// Minimal IP address representation (IPv4).
#[derive(Debug, Clone, Copy)]
pub struct IpAddr(pub [u8; 4]);

impl core::fmt::Display for IpAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let [a, b, c, d] = self.0;
        write!(f, "{a}.{b}.{c}.{d}")
    }
}

pub trait WifiDriver: Send + Sync {
    fn connect(&mut self, ssid: &str, password: &str) -> Result<(), WifiError>;
    fn disconnect(&mut self) -> Result<(), WifiError>;
    fn is_connected(&self) -> bool;
    fn get_ip(&self) -> Option<IpAddr>;
    /// Signal strength in dBm.
    fn get_signal_strength(&self) -> i8;
}
