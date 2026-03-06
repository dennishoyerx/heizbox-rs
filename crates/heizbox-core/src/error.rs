use thiserror::Error;

// ── Domain errors ────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Heater error: {0}")]
    Heater(#[from] HeaterError),
    #[error("Sensor error: {0}")]
    Sensor(#[from] SensorError),
    #[error("Calibration failed: {0}")]
    Calibration(String),
    #[error("Invalid state transition")]
    InvalidStateTransition,
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, Error)]
pub enum HeaterError {
    #[error("Cutoff temperature {0}°C exceeded limit {1}°C")]
    CutoffExceeded(u16, u16),
    #[error("Cycle timeout exceeded: {0} ms")]
    CycleTimeoutExceeded(u32),
    #[error("Invalid temperature reading")]
    InvalidTemperature,
    #[error("Heater not initialised")]
    NotInitialized,
    #[error("Invalid state transition")]
    InvalidStateTransition,
}

#[derive(Debug, Error)]
pub enum SensorError {
    #[error("I2C communication failed")]
    I2cFailed,
    #[error("SPI communication failed")]
    SpiFailed,
    #[error("Sensor not initialised")]
    NotInitialized,
    #[error("Invalid calibration data")]
    InvalidCalibration,
    #[error("Sensor not responding")]
    NoResponse,
}

// ── HAL / infrastructure errors ──────────────────────────────────────────────

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

#[derive(Debug, Error)]
pub enum GpioError {
    #[error("Invalid pin {0}")]
    InvalidPin(u8),
    #[error("Pin not configured")]
    NotConfigured,
    #[error("Hardware error")]
    HardwareError,
}

#[derive(Debug, Error)]
pub enum I2cError {
    #[error("I2C bus error")]
    BusError,
    #[error("Address NAK for 0x{0:02x}")]
    AddressNak(u8),
    #[error("Data NAK")]
    DataNak,
    #[error("Timeout")]
    Timeout,
}

#[derive(Debug, Error)]
pub enum SpiError {
    #[error("SPI bus error")]
    BusError,
    #[error("Chip-select error")]
    ChipSelectError,
    #[error("Timeout")]
    Timeout,
}

#[derive(Debug, Error)]
pub enum AdcError {
    #[error("ADC channel error for pin {0}")]
    ChannelError(u8),
    #[error("ADC conversion error")]
    ConversionError,
    #[error("Not ready")]
    NotReady,
}

#[derive(Debug, Error)]
pub enum TimerError {
    #[error("Timer already running")]
    AlreadyRunning,
    #[error("Timer not running")]
    NotRunning,
    #[error("Invalid channel")]
    InvalidChannel,
}

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

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("TLS error")]
    TlsError,
    #[error("HTTP error: {0}")]
    HttpError(u16),
    #[error("DNS resolution failed")]
    DnsError,
    #[error("Connection timeout")]
    Timeout,
    #[error("Already connecting")]
    AlreadyConnecting,
    #[error("Reconnect failed after retries")]
    ReconnectFailed,
    #[error("Not connected")]
    NotConnected,
    #[error("Receive error")]
    ReceiveError,
    #[error("Serialisation error")]
    SerializationError,
}

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("NVS operation failed")]
    NvsError,
    #[error("Deserialisation failed: {0}")]
    DeserializationError(String),
}

#[derive(Debug, Error)]
pub enum HalError {
    #[error("NVS error: {0}")]
    Nvs(#[from] NvsError),
    #[error("GPIO error: {0}")]
    Gpio(#[from] GpioError),
    #[error("I2C error: {0}")]
    I2c(#[from] I2cError),
    #[error("SPI error: {0}")]
    Spi(#[from] SpiError),
    #[error("ADC error: {0}")]
    Adc(#[from] AdcError),
    #[error("Timer error: {0}")]
    Timer(#[from] TimerError),
    #[error("WiFi error: {0}")]
    Wifi(#[from] WifiError),
}

#[derive(Debug, Error)]
pub enum InfraError {
    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    #[error("HAL error: {0}")]
    Hal(#[from] HalError),
}

// ── Convenience aliases ───────────────────────────────────────────────────────

pub type DomainResult<T> = Result<T, DomainError>;
pub type InfraResult<T> = Result<T, InfraError>;

// ── From conversions between NvsError ────────────────────────────────────────

impl From<NvsError> for PersistenceError {
    fn from(_: NvsError) -> Self {
        PersistenceError::NvsError
    }
}
