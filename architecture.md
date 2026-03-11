# Rust Firmware Architecture: Heizbox ESP32 Induktionsheizer

## 1. ARCHITEKTUR-ÜBERSICHT

### 1.1 Architekturprinzipien

```
┌─────────────────────────────────────────────────────┐
│              APPLICATION LAYER                       │
│  (Screens, Navigation, Input Handling)              │
├─────────────────────────────────────────────────────┤
│              DOMAIN LAYER                            │
│  (Heater SM, Events, Sensors, Persistence Model)    │
├─────────────────────────────────────────────────────┤
│          INFRASTRUCTURE LAYER                        │
│  (Repos, Network Client, Clock, OTA Service)        │
├─────────────────────────────────────────────────────┤
│              HAL LAYER                               │
│  (Driver Traits: GPIO, SPI, I2C, WebSocket)         │
├─────────────────────────────────────────────────────┤
│         ESP32 HARDWARE / esp-idf-sys                │
└─────────────────────────────────────────────────────┘
```

**Invarianten:**
- Domain ↔ Application bidirektional
- Infrastructure → Domain nur via Traits
- HAL ← Application nur Trait-based
- Keine zirkulären Dependencies
- Keine Global States außer Singletons mit `OnceLock`

---

## 2. PROJEKTSTRUKTUR (CARGO WORKSPACE)

```
heizbox-rs/
├── Cargo.toml (workspace)
├── crates/
│   ├── heizbox-core/           # Domain Layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── device.rs       # Root Aggregate
│   │   │   ├── heater/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── state.rs    # State Machine
│   │   │   │   ├── cycle.rs    # Heat Cycle Model
│   │   │   │   ├── safety.rs   # Safety Checks
│   │   │   │   ├── temperature.rs
│   │   │   │   └── calibration.rs
│   │   │   ├── consumption/
│   │   │   │   ├── mod.rs
│   │   │   │   └── model.rs
│   │   │   ├── event/
│   │   │   │   ├── mod.rs
│   │   │   │   └── types.rs    # DomainEvent enum
│   │   │   ├── error.rs        # Domain Errors
│   │   │   └── config.rs       # Constants
│   │   └── Cargo.toml
│   │
│   ├── heizbox-app/            # Application Layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── device.rs       # DeviceApp (orchestration)
│   │   │   ├── screen/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── state.rs
│   │   │   │   ├── nav.rs      # Navigation FSM
│   │   │   │   ├── fire.rs
│   │   │   │   ├── menu.rs
│   │   │   │   └── startup.rs
│   │   │   ├── input/
│   │   │   │   ├── mod.rs
│   │   │   │   └── handler.rs
│   │   │   ├── event_bus.rs    # Application EventBus
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   ├── heizbox-infra/          # Infrastructure Layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── persistence/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── nvs_repo.rs # NVS Repository
│   │   │   │   └── models.rs
│   │   │   ├── network/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── http_client.rs
│   │   │   │   ├── ws_client.rs
│   │   │   │   ├── heartbeat.rs
│   │   │   │   └── reconnect.rs
│   │   │   ├── clock/
│   │   │   │   ├── mod.rs
│   │   │   │   └── ntp.rs
│   │   │   ├── ota/
│   │   │   │   ├── mod.rs
│   │   │   │   └── service.rs
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   ├── heizbox-hal/            # HAL Abstraction
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── gpio.rs         # GPIO Trait
│   │   │   ├── spi.rs          # SPI Trait
│   │   │   ├── i2c.rs          # I2C Trait
│   │   │   ├── timer.rs        # Timer Trait
│   │   │   ├── adc.rs          # ADC Trait
│   │   │   ├── nvs.rs          # NVS Trait
│   │   │   ├── wifi.rs         # WiFi Trait
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   ├── heizbox-esp32/          # ESP32 Implementation
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── impl/
│   │   │   │   ├── gpio_impl.rs
│   │   │   │   ├── spi_impl.rs
│   │   │   │   ├── i2c_impl.rs
│   │   │   │   ├── nvs_impl.rs
│   │   │   │   ├── wifi_impl.rs
│   │   │   │   └── adc_impl.rs
│   │   │   ├── config.rs       # Pin config
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   └── heizbox-tests/          # Integration Tests
│       ├── tests/
│       │   ├── heater_sm.rs
│       │   ├── network.rs
│       │   └── persistence.rs
│       └── Cargo.toml
│
└── docs/
    ├── architecture.md
    ├── migration.md
    └── safety_analysis.md
```

---

## 3. DOMAIN MODEL (heizbox-core)

### 3.1 Heater State Machine (typestate pattern)

```rust
// crates/heizbox-core/src/heater/state.rs

use core::marker::PhantomData;

/// State Machine Marker Types
pub struct Idle;
pub struct Heating;
pub struct Paused;
pub struct Error;

/// Heater state is encoded in the type system
pub struct HeaterSm<State = Idle> {
    power: u8,                          // 0-100%
    target_temp: u16,                   // °C
    current_temp: u16,                  // °C
    auto_stop_time_ms: u32,
    cycle_duration_ms: u32,
    cycle_started_at: Option<u32>,
    ir_calibration: IrCalibration,
    _state: PhantomData<State>,
}

impl HeaterSm<Idle> {
    pub fn new(config: HeaterConfig) -> Self {
        Self {
            power: config.power,
            target_temp: config.target_temp,
            current_temp: 0,
            auto_stop_time_ms: config.auto_stop_time_ms,
            cycle_duration_ms: 0,
            cycle_started_at: None,
            ir_calibration: IrCalibration::default(),
            _state: PhantomData,
        }
    }

    /// Transition: Idle → Heating
    pub fn start_heating(mut self, cycle_started_at: u32) 
        -> Result<HeaterSm<Heating>, HeaterError> 
    {
        self.cycle_started_at = Some(cycle_started_at);
        Ok(HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: 0,
            cycle_started_at: self.cycle_started_at,
            ir_calibration: self.ir_calibration,
            _state: PhantomData,
        })
    }
}

impl HeaterSm<Heating> {
    /// Update temperature and check safety
    pub fn update_temperature(
        mut self,
        new_temp: u16,
        now_ms: u32,
    ) -> Result<HeaterSm<Heating>, HeaterError> {
        self.current_temp = new_temp;
        self.cycle_duration_ms = now_ms - self.cycle_started_at.unwrap_or(0);

        // Safety checks
        if self.check_cutoff_exceeded()? {
            return Err(HeaterError::CutoffTemperatureExceeded);
        }
        if self.check_timeout_exceeded()? {
            return Err(HeaterError::CycleTimeoutExceeded);
        }

        Ok(self)
    }

    /// Check if target reached
    pub fn is_target_reached(&self) -> bool {
        self.current_temp >= self.target_temp
    }

    /// Transition: Heating → Paused
    pub fn pause(self) -> HeaterSm<Paused> {
        HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: self.cycle_duration_ms,
            cycle_started_at: self.cycle_started_at,
            ir_calibration: self.ir_calibration,
            _state: PhantomData,
        }
    }

    fn check_cutoff_exceeded(&self) -> Result<bool, HeaterError> {
        if self.target_temp == 420 {
            return Ok(false); // 420 = disabled
        }
        Ok(self.current_temp > self.target_temp + 20)
    }

    fn check_timeout_exceeded(&self) -> Result<bool, HeaterError> {
        Ok(self.cycle_duration_ms > self.auto_stop_time_ms)
    }
}

impl HeaterSm<Paused> {
    /// Transition: Paused → Heating
    pub fn resume(self, now_ms: u32) -> HeaterSm<Heating> {
        HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: self.cycle_duration_ms,
            cycle_started_at: Some(now_ms - self.cycle_duration_ms),
            ir_calibration: self.ir_calibration,
            _state: PhantomData,
        }
    }

    /// Transition: Paused → Idle (finalize)
    pub fn finalize(self) -> (HeaterSm<Idle>, CycleResult) {
        let result = CycleResult {
            duration_ms: self.cycle_duration_ms,
            max_temp: self.current_temp,
            started_at: self.cycle_started_at,
        };
        let idle = HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: 0,
            cycle_started_at: None,
            ir_calibration: self.ir_calibration,
            _state: PhantomData,
        };
        (idle, result)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CycleResult {
    pub duration_ms: u32,
    pub max_temp: u16,
    pub started_at: Option<u32>,
}

#[derive(Debug)]
pub enum HeaterError {
    CutoffTemperatureExceeded,
    CycleTimeoutExceeded,
    InvalidTemperatureReading,
    CalibrationFailed,
}
```

**Vorteile des typestate Ansatzes:**
- ✅ Unmögliche Zustände sind nicht repräsentierbar
- ✅ Zustandsübergänge sind kompilzeitlich sicher
- ✅ Keine Runtime State Checks nötig
- ✅ Zero-cost abstraction (wird wegoptimiert)
- ❌ Etwas boilerplate-reich

### 3.2 Domain Events

```rust
// crates/heizbox-core/src/event/types.rs

use heizbox_core::heater::CycleResult;

#[derive(Debug, Clone)]
pub enum DomainEvent {
    // Heater events
    HeatingStarted {
        target_temp: u16,
        timestamp_ms: u32,
    },
    HeatingPaused {
        current_temp: u16,
        duration_ms: u32,
    },
    CycleFinished(CycleResult),
    HeatingError(HeaterErrorEvent),
    
    // Temperature events
    TemperatureUpdated {
        current: u16,
        ambient: u16,
        raw_ir: u16,
    },
    
    // Network events
    WifiConnected { ssid: String },
    WifiDisconnected { reason: u8 },
    WebSocketConnected,
    WebSocketDisconnected,
    
    // Persistence events
    SettingsPersisted { key: &'static str },
    
    // OTA events
    OtaStarted,
    OtaProgress { percent: u8 },
    OtaCompleted,
    OtaFailed { reason: &'static str },
    
    // UI events
    ScreenChanged { screen: ScreenType },
    InputReceived(InputEvent),
}

#[derive(Debug, Clone)]
pub enum HeaterErrorEvent {
    CutoffExceeded { temp: u16, limit: u16 },
    TimeoutExceeded { duration: u32, limit: u32 },
    InvalidReading { reason: &'static str },
}
```

### 3.3 Domain Errors

```rust
// crates/heizbox-core/src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Heater error: {0}")]
    Heater(#[from] HeaterError),

    #[error("Temperature sensor error: {0}")]
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

    #[error("Cycle timeout exceeded: {0}ms")]
    CycleTimeoutExceeded(u32),

    #[error("Invalid temperature reading")]
    InvalidTemperature,
}

#[derive(Debug, Error)]
pub enum SensorError {
    #[error("I2C communication failed")]
    I2cFailed,

    #[error("SPI communication failed")]
    SpiFailed,

    #[error("Sensor not initialized")]
    NotInitialized,

    #[error("Invalid calibration data")]
    InvalidCalibration,
}
```

---

## 4. CONCURRENCY MODELL

### Entscheidung: **Single-Threaded Event Loop mit Embassy async**

**Begründung:**
1. **Ressourcen:** ESP32 hat nur 2 Cores, aber ausgelastet durch WiFi Stack
2. **Deterministik:** Heat-Control braucht garantierte Response Times
3. **Komplexität:** Multithread auf Embedded = zusätzliche Fehlerquellen
4. **Embassy async:** Native FreeRTOS Tasks unter der Haube, aber Rust-native Syntax
5. **Message Passing:** Decoupling via async channels statt shared state

**Alternative betrachtet: std + FreeRTOS**
- ❌ Höhere Komplexität (Locks, Mutexes)
- ❌ Race Conditions möglich
- ✅ Etwas mehr Parallelismus
- ❌ Aber: WiFi Stack ist ohnehin single-threaded

**Modell:**

```
┌─────────────────────────────────────────┐
│     EMBASSY RUNTIME (FreeRTOS)          │
│                                         │
│  ┌────────────────────────────────────┐ │
│  │  Main Control Task (Priority 5)    │ │  ← Heater SM, Sensor Updates
│  │  (core: 0)                         │ │
│  └────────────────────────────────────┘ │
│           ↓                              │
│  ┌────────────────────────────────────┐ │
│  │  Network Task (Priority 3)         │ │  ← WiFi, WS, HTTP
│  │  (core: 1)                         │ │
│  └────────────────────────────────────┘ │
│           ↓                              │
│  ┌────────────────────────────────────┐ │
│  │  UI/Display Task (Priority 4)      │ │  ← Screen Render
│  │  (core: 0)                         │ │
│  └────────────────────────────────────┘ │
│           ↓                              │
│  ┌────────────────────────────────────┐ │
│  │  Input Task (Priority 2)           │ │  ← Joystick polling
│  │  (core: 0)                         │ │
│  └────────────────────────────────────┘ │
│                                         │
│  ─────── async channels ───────        │
│                                         │
└─────────────────────────────────────────┘
```

### 4.1 Event Bus Implementation (async channel-based)

```rust
// crates/heizbox-app/src/event_bus.rs

use embassy_sync::channel::{Channel, Sender, Receiver};
use heizbox_core::event::DomainEvent;

pub type EventBusChannel = Channel<embassy_executor::Spawner, DomainEvent, 16>;

pub struct EventBus {
    tx: Sender<'static, embassy_executor::Spawner, DomainEvent, 16>,
    rx: Receiver<'static, embassy_executor::Spawner, DomainEvent, 16>,
}

impl EventBus {
    pub fn new(channel: &'static EventBusChannel) -> (EventBus, EventBus) {
        let (tx, rx) = channel.split();
        (
            EventBus { tx: tx.clone(), rx },
            EventBus { tx, rx: rx.clone() },
        )
    }

    pub async fn publish(&mut self, event: DomainEvent) {
        let _ = self.tx.send(event).await;
    }

    pub async fn subscribe(&mut self) -> DomainEvent {
        self.rx.recv().await
    }
}

/// Static EventBus Channel
pub static EVENT_BUS: EventBusChannel = Channel::new();

/// Event Subscribers
pub struct EventSubscribers {
    pub heater_rx: Receiver<'static, embassy_executor::Spawner, DomainEvent, 16>,
    pub ui_rx: Receiver<'static, embassy_executor::Spawner, DomainEvent, 16>,
    pub network_rx: Receiver<'static, embassy_executor::Spawner, DomainEvent, 16>,
}
```

### 4.2 Task Orchestration

```rust
// crates/heizbox-esp32/src/main.rs

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemParts::take();
    
    // Initialize HAL
    let gpio = GpioImpl::new(peripherals.gpio);
    let i2c = I2cImpl::new(peripherals.i2c0);
    let spi = SpiImpl::new(peripherals.spi3);
    let adc = AdcImpl::new(peripherals.adc1);
    let nvs = NvsImpl::new();
    let wifi = WifiImpl::new(peripherals.modem);
    
    // Create app instance
    let mut app = DeviceApp::new(
        Box::new(gpio),
        Box::new(i2c),
        Box::new(spi),
        Box::new(nvs),
        Box::new(wifi),
    );
    
    // Spawn main control task
    spawner.spawn(control_task(app.clone())).ok();
    
    // Spawn network task
    spawner.spawn(network_task(app.clone())).ok();
    
    // Spawn UI task
    spawner.spawn(ui_task(app.clone())).ok();
    
    // Spawn input task
    spawner.spawn(input_task(app.clone())).ok();
    
    // Spawn heartbeat task
    spawner.spawn(heartbeat_task(app.clone())).ok();
}

#[embassy_executor::task]
async fn control_task(mut app: DeviceApp) {
    let mut interval = Ticker::every(Duration::from_millis(100));
    
    loop {
        // Update heater state machine
        app.update_heater().await;
        
        // Read temperature sensors
        app.update_sensors().await;
        
        // Publish domain events
        if let Some(event) = app.get_pending_events() {
            app.publish_event(event).await;
        }
        
        interval.next().await;
    }
}

#[embassy_executor::task]
async fn network_task(mut app: DeviceApp) {
    app.network_loop().await;
}

#[embassy_executor::task]
async fn ui_task(mut app: DeviceApp) {
    loop {
        app.render_screen().await;
        Timer::after(Duration::from_millis(50)).await;
    }
}

#[embassy_executor::task]
async fn input_task(mut app: DeviceApp) {
    let mut interval = Ticker::every(Duration::from_millis(50));
    
    loop {
        if let Some(event) = app.read_input().await {
            app.handle_input_event(event).await;
        }
        interval.next().await;
    }
}
```

---

## 5. INFRASTRUCTURE LAYER

### 5.1 NVS Repository Pattern

```rust
// crates/heizbox-infra/src/persistence/nvs_repo.rs

use heizbox_core::heater::HeaterConfig;
use heizbox_hal::nvs::NvsDriver;

#[derive(Clone)]
pub struct HeaterConfigRepository {
    nvs: Arc<Box<dyn NvsDriver>>,
}

impl HeaterConfigRepository {
    pub fn new(nvs: Arc<Box<dyn NvsDriver>>) -> Self {
        Self { nvs }
    }

    pub async fn load(&self) -> Result<HeaterConfig, RepositoryError> {
        let power = self.nvs.get_u8("heater", "power", 100)?;
        let target_temp = self.nvs.get_u16("heater", "target_temp", 200)?;
        let auto_stop_ms = self.nvs.get_u32("heater", "auto_stop_ms", 90000)?;

        Ok(HeaterConfig {
            power,
            target_temp,
            auto_stop_time_ms: auto_stop_ms,
            ..Default::default()
        })
    }

    pub async fn save(&self, config: &HeaterConfig) -> Result<(), RepositoryError> {
        self.nvs.set_u8("heater", "power", config.power)?;
        self.nvs.set_u16("heater", "target_temp", config.target_temp)?;
        self.nvs.set_u32("heater", "auto_stop_ms", config.auto_stop_time_ms)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("NVS operation failed")]
    NvsError,
    #[error("Deserialization failed: {0}")]
    DeserializationError(String),
}
```

### 5.2 Network Client (HTTP + WebSocket)

```rust
// crates/heizbox-infra/src/network/http_client.rs

use embassy_net::dns::DnsSocket;

pub struct HttpClient {
    dns: DnsSocket<'static>,
    tls_config: TlsConfig, // cert pinning
}

impl HttpClient {
    pub async fn post_stats(
        &mut self,
        stats: &DeviceStats,
    ) -> Result<(), NetworkError> {
        let url = "https://backend.hzbx.de/stats";
        let json = serde_json::to_string(stats)?;
        
        let response = self.tls_post(url, &json).await?;
        
        match response.status {
            200..=299 => Ok(()),
            _ => Err(NetworkError::HttpError(response.status)),
        }
    }

    async fn tls_post(
        &mut self,
        url: &str,
        body: &str,
    ) -> Result<HttpResponse, NetworkError> {
        // Use mbedtls with cert pinning
        let socket = TlsSocket::connect(url, self.tls_config.clone()).await?;
        let response = socket.post(url, body).await?;
        Ok(response)
    }
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
}
```

### 5.3 WebSocket Client mit Auto-Reconnect

```rust
// crates/heizbox-infra/src/network/ws_client.rs

use async_trait::async_trait;
use heizbox_core::event::DomainEvent;

pub struct WebSocketClient {
    url: String,
    socket: Option<WebSocket>,
    state: WsState,
    reconnect_strategy: ExponentialBackoff,
}

#[derive(Clone, Copy, Debug)]
enum WsState {
    Disconnected,
    Connecting,
    Connected,
    Failed(u32), // retry count
}

#[async_trait]
impl WebSocketClient {
    pub async fn connect(&mut self) -> Result<(), NetworkError> {
        match self.state {
            WsState::Connected => return Ok(()),
            WsState::Connecting => return Err(NetworkError::AlreadyConnecting),
            _ => {}
        }

        self.state = WsState::Connecting;

        loop {
            match self.attempt_connect().await {
                Ok(ws) => {
                    self.socket = Some(ws);
                    self.state = WsState::Connected;
                    self.reconnect_strategy.reset();
                    return Ok(());
                }
                Err(e) => {
                    let retry_count = match self.state {
                        WsState::Failed(n) => n + 1,
                        _ => 1,
                    };
                    self.state = WsState::Failed(retry_count);

                    if retry_count > 10 {
                        return Err(NetworkError::ReconnectFailed);
                    }

                    let backoff = self.reconnect_strategy.next_backoff();
                    Timer::after(Duration::from_millis(backoff)).await;
                }
            }
        }
    }

    async fn attempt_connect(&mut self) -> Result<WebSocket, NetworkError> {
        // Non-blocking connect with timeout
        timeout(
            Duration::from_secs(10),
            WebSocket::connect(&self.url),
        )
        .await
        .map_err(|_| NetworkError::Timeout)?
    }

    pub async fn send(&mut self, event: DomainEvent) -> Result<(), NetworkError> {
        if !matches!(self.state, WsState::Connected) {
            self.connect().await?;
        }

        let json = serde_json::to_string(&event)?;
        self.socket
            .as_mut()
            .ok_or(NetworkError::NotConnected)?
            .send(json.as_bytes())
            .await?;

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, NetworkError> {
        if !matches!(self.state, WsState::Connected) {
            return Err(NetworkError::NotConnected);
        }

        self.socket
            .as_mut()
            .ok_or(NetworkError::NotConnected)?
            .recv()
            .await
            .map_err(|_| NetworkError::ReceiveError)
    }
}

pub struct ExponentialBackoff {
    current: u32,
    max: u32,
}

impl ExponentialBackoff {
    pub fn new() -> Self {
        Self {
            current: 100,
            max: 30000,
        }
    }

    pub fn next_backoff(&mut self) -> u32 {
        self.current = core::cmp::min(self.current * 2, self.max);
        self.current
    }

    pub fn reset(&mut self) {
        self.current = 100;
    }
}
```

---

## 6. HAL ABSTRACTION LAYER

```rust
// crates/heizbox-hal/src/lib.rs

pub mod gpio;
pub mod i2c;
pub mod spi;
pub mod nvs;
pub mod adc;
pub mod timer;
pub mod wifi;

pub use gpio::GpioDriver;
pub use i2c::I2cDriver;
pub use spi::SpiDriver;
pub use nvs::NvsDriver;
pub use adc::AdcDriver;
pub use timer::TimerDriver;
pub use wifi::WifiDriver;

// NVS Trait
pub trait NvsDriver: Send + Sync {
    fn get_u8(&self, namespace: &str, key: &str, default: u8) -> Result<u8, NvsError>;
    fn get_u16(&self, namespace: &str, key: &str, default: u16) -> Result<u16, NvsError>;
    fn get_u32(&self, namespace: &str, key: &str, default: u32) -> Result<u32, NvsError>;
    fn get_i32(&self, namespace: &str, key: &str, default: i32) -> Result<i32, NvsError>;
    fn get_f32(&self, namespace: &str, key: &str, default: f32) -> Result<f32, NvsError>;
    fn get_str(&self, namespace: &str, key: &str) -> Result<String, NvsError>;
    
    fn set_u8(&self, namespace: &str, key: &str, value: u8) -> Result<(), NvsError>;
    fn set_u16(&self, namespace: &str, key: &str, value: u16) -> Result<(), NvsError>;
    fn set_u32(&self, namespace: &str, key: &str, value: u32) -> Result<(), NvsError>;
    fn set_i32(&self, namespace: &str, key: &str, value: i32) -> Result<(), NvsError>;
    fn set_f32(&self, namespace: &str, key: &str, value: f32) -> Result<(), NvsError>;
    fn set_str(&self, namespace: &str, key: &str, value: &str) -> Result<(), NvsError>;
    
    fn erase(&self, namespace: &str) -> Result<(), NvsError>;
}

#[derive(Debug, Error)]
pub enum NvsError {
    #[error("Key not found")]
    KeyNotFound,
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("NVS full")]
    NvsFull,
    #[error("Invalid value")]
    InvalidValue,
}

// GPIO Trait
pub trait GpioDriver: Send + Sync {
    fn set_output(&mut self, pin: u8) -> Result<(), GpioError>;
    fn set_input(&mut self, pin: u8) -> Result<(), GpioError>;
    fn write(&mut self, pin: u8, level: bool) -> Result<(), GpioError>;
    fn read(&self, pin: u8) -> Result<bool, GpioError>;
}

#[derive(Debug, Error)]
pub enum GpioError {
    #[error("Invalid pin")]
    InvalidPin,
    #[error("Pin already in use")]
    PinInUse,
}

// I2C Trait
pub trait I2cDriver: Send + Sync {
    async fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), I2cError>;
    async fn read(&mut self, addr: u8, len: usize) -> Result<Vec<u8>, I2cError>;
    async fn write_read(
        &mut self,
        addr: u8,
        write: &[u8],
        read_len: usize,
    ) -> Result<Vec<u8>, I2cError>;
}

#[derive(Debug, Error)]
pub enum I2cError {
    #[error("Bus error")]
    BusError,
    #[error("Address NAK")]
    AddressNak,
    #[error("Data NAK")]
    DataNak,
    #[error("Timeout")]
    Timeout,
}

// WiFi Trait
pub trait WifiDriver: Send + Sync {
    async fn connect(&mut self, ssid: &str, password: &str) -> Result<(), WifiError>;
    async fn disconnect(&mut self) -> Result<(), WifiError>;
    fn is_connected(&self) -> bool;
    fn get_ip(&self) -> Option<IpAddr>;
    fn get_signal_strength(&self) -> i8; // dBm
}

#[derive(Debug, Error)]
pub enum WifiError {
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Connection timeout")]
    Timeout,
    #[error("Invalid SSID")]
    InvalidSsid,
}
```

---

## 7. APPLICATION LAYER

### 7.1 Screen Navigation FSM

```rust
// crates/heizbox-app/src/screen/nav.rs

use crate::screen::state::ScreenState;

pub struct NavigationFsm {
    current: ScreenType,
    previous: ScreenType,
    history: heapless::Vec<ScreenType, 8>,
}

impl NavigationFsm {
    pub fn new() -> Self {
        Self {
            current: ScreenType::Startup,
            previous: ScreenType::Startup,
            history: heapless::Vec::new(),
        }
    }

    pub fn navigate_to(&mut self, target: ScreenType) -> Result<(), NavError> {
        // Validate transition
        self.validate_transition(self.current, target)?;
        
        // Push to history
        self.history.push(self.current).ok();
        
        // Update state
        self.previous = self.current;
        self.current = target;
        
        Ok(())
    }

    pub fn navigate_back(&mut self) -> Result<(), NavError> {
        if let Some(previous) = self.history.pop() {
            self.current = previous;
            Ok(())
        } else {
            Err(NavError::NoHistory)
        }
    }

    fn validate_transition(&self, from: ScreenType, to: ScreenType) -> Result<(), NavError> {
        let valid_transitions = match from {
            ScreenType::Startup => &[ScreenType::Fire][..],
            ScreenType::Fire => &[
                ScreenType::Fire,
                ScreenType::Menu,
                ScreenType::Screensaver,
            ][..],
            ScreenType::Menu => &[ScreenType::Fire][..],
            ScreenType::Screensaver => &[ScreenType::Fire][..],
            ScreenType::OtaUpdate => &[ScreenType::Fire][..],
            _ => &[][..],
        };

        if valid_transitions.contains(&to) {
            Ok(())
        } else {
            Err(NavError::InvalidTransition(from, to))
        }
    }

    pub fn current_screen(&self) -> ScreenType {
        self.current
    }
}

#[derive(Debug)]
pub enum NavError {
    InvalidTransition(ScreenType, ScreenType),
    NoHistory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenType {
    Startup,
    Fire,
    Menu,
    Screensaver,
    OtaUpdate,
}
```

### 7.2 Screen Trait

```rust
// crates/heizbox-app/src/screen/mod.rs

use embassy_sync::channel::Receiver;
use heizbox_core::event::DomainEvent;

#[async_trait::async_trait]
pub trait Screen: Send {
    async fn on_enter(&mut self);
    async fn on_exit(&mut self);
    async fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError>;
    async fn update(&mut self, event: DomainEvent) -> Result<(), ScreenError>;
    async fn render(&self) -> Result<FrameBuffer, ScreenError>;
}

#[derive(Debug)]
pub enum Navigation {
    None,
    GoTo(ScreenType),
    Back,
    Exit,
}

#[derive(Debug, Error)]
pub enum ScreenError {
    #[error("Render failed")]
    RenderError,
    #[error("Event handling failed")]
    EventError,
}

pub struct FrameBuffer {
    pub data: heapless::Vec<u8, 65536>, // 280x240 16-bit color
    pub width: u16,
    pub height: u16,
}

// Concrete Screen Implementations
pub struct FireScreen {
    heater_state: HeaterSmState,
    menu: MenuState,
    overlay: Option<OverlayMessage>,
    consumption: ConsumptionDisplay,
}

#[async_trait::async_trait]
impl Screen for FireScreen {
    async fn on_enter(&mut self) {
        // Initialize screen state
    }

    async fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError> {
        match event.button {
            Button::Fire => {
                // Toggle heating
                Ok(Navigation::None)
            }
            Button::Menu => Ok(Navigation::GoTo(ScreenType::Menu)),
            _ => Ok(Navigation::None),
        }
    }

    async fn update(&mut self, event: DomainEvent) -> Result<(), ScreenError> {
        match event {
            DomainEvent::TemperatureUpdated { current, .. } => {
                self.heater_state.current_temp = current;
            }
            DomainEvent::CycleFinished(result) => {
                self.overlay = Some(OverlayMessage::new("Cycle finished"));
            }
            _ => {}
        }
        Ok(())
    }

    async fn render(&self) -> Result<FrameBuffer, ScreenError> {
        let mut fb = FrameBuffer::new(280, 240);
        
        // Render temperature gradient background
        let gradient_color = self.get_color_for_temp(self.heater_state.current_temp);
        self.render_gradient_bg(&mut fb, gradient_color)?;
        
        // Render temperature display
        self.render_temperature(&mut fb)?;
        
        // Render consumption stats
        self.render_consumption(&mut fb)?;
        
        // Render overlay if present
        if let Some(overlay) = &self.overlay {
            self.render_overlay(&mut fb, overlay)?;
        }

        Ok(fb)
    }
}

impl FireScreen {
    fn get_color_for_temp(&self, temp: u16) -> Rgb565 {
        match temp {
            0..=165 => Rgb565::from((30, 202, 211)), // Cyan (cold)
            166..=180 => Rgb565::from((46, 204, 113)), // Green (flavor)
            181..=195 => Rgb565::from((241, 196, 15)), // Yellow (balanced)
            196..=215 => Rgb565::from((230, 126, 34)), // Orange (extraction)
            _ => Rgb565::from((192, 57, 43)), // Red (hot)
        }
    }
}
```

### 7.3 Input Handler (state machine für Input Events)

```rust
// crates/heizbox-app/src/input/handler.rs

pub struct InputHandler {
    state: InputHandlerState,
    button_states: [ButtonState; 6],
    last_input_ms: u32,
}

#[derive(Clone, Copy)]
enum InputHandlerState {
    Normal,
    LongPressActive,
    MenuMode,
}

#[derive(Clone, Copy, Default)]
struct ButtonState {
    pressed_at: u32,
    is_pressed: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            state: InputHandlerState::Normal,
            button_states: [ButtonState::default(); 6],
            last_input_ms: 0,
        }
    }

    pub fn handle_input(
        &mut self,
        button: Button,
        now_ms: u32,
        is_pressed: bool,
    ) -> Result<Option<InputEvent>, InputError> {
        self.last_input_ms = now_ms;

        match self.state {
            InputHandlerState::Normal => {
                self.handle_normal_input(button, now_ms, is_pressed)
            }
            InputHandlerState::LongPressActive => {
                self.handle_longpress_input(button, now_ms, is_pressed)
            }
            InputHandlerState::MenuMode => {
                self.handle_menu_input(button, now_ms, is_pressed)
            }
        }
    }

    fn handle_normal_input(
        &mut self,
        button: Button,
        now_ms: u32,
        is_pressed: bool,
    ) -> Result<Option<InputEvent>, InputError> {
        if is_pressed {
            let button_idx = button as usize;
            self.button_states[button_idx].pressed_at = now_ms;
            self.button_states[button_idx].is_pressed = true;
            Ok(None) // Wait for release or timeout
        } else {
            let button_idx = button as usize;
            let state = &mut self.button_states[button_idx];
            let hold_duration = now_ms - state.pressed_at;
            state.is_pressed = false;

            if hold_duration > 300 {
                // Long press detected
                self.state = InputHandlerState::LongPressActive;
                Ok(Some(InputEvent {
                    button,
                    event_type: InputEventType::LongPress,
                }))
            } else {
                // Short press
                Ok(Some(InputEvent {
                    button,
                    event_type: InputEventType::Press,
                }))
            }
        }
    }

    fn handle_longpress_input(
        &mut self,
        button: Button,
        now_ms: u32,
        is_pressed: bool,
    ) -> Result<Option<InputEvent>, InputError> {
        if !is_pressed {
            self.state = InputHandlerState::Normal;
            Ok(Some(InputEvent {
                button,
                event_type: InputEventType::Release,
            }))
        } else {
            Ok(None)
        }
    }

    fn handle_menu_input(
        &mut self,
        button: Button,
        _now_ms: u32,
        is_pressed: bool,
    ) -> Result<Option<InputEvent>, InputError> {
        if is_pressed {
            match button {
                Button::Up => Ok(Some(InputEvent {
                    button,
                    event_type: InputEventType::Press,
                })),
                Button::Down => Ok(Some(InputEvent {
                    button,
                    event_type: InputEventType::Press,
                })),
                Button::Left => {
                    self.state = InputHandlerState::Normal;
                    Ok(Some(InputEvent {
                        button,
                        event_type: InputEventType::Release,
                    }))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_last_input_ms(&self) -> u32 {
        self.last_input_ms
    }
}

#[derive(Debug, Clone)]
pub struct InputEvent {
    pub button: Button,
    pub event_type: InputEventType,
}

#[derive(Debug, Clone, Copy)]
pub enum InputEventType {
    Press,
    LongPress,
    Release,
    Hold,
}

#[derive(Debug, Clone, Copy)]
pub enum Button {
    Fire,
    Up,
    Down,
    Left,
    Right,
    Center,
}
```

---

## 8. PERSISTENCE ARCHITECTURE

### 8.1 Settings Model

```rust
// crates/heizbox-core/src/persistence/model.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaterSettings {
    pub power: u8,
    pub mode: HeatingMode,
    pub presets: [u16; 4], // Flavor, Balanced, Extraction, Full
    pub ir_emissivity: u8,
    pub ir_correction: i8,
    pub temp_sensor_read_interval: u16,
    pub temp_sensor_off_time: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub brightness: u8,
    pub idle_brightness: u8,
    pub idle_timeout_minutes: u16,
    pub dark_mode: bool,
    pub flip_orientation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSettings {
    pub timezone_offset_seconds: i32,
    pub sleep_timeout_ms: u32,
    pub wifi_ssid: String, // Optional, not stored
    pub wifi_password: String, // Optional, not stored
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HeatingMode {
    Temperature,
    Preset,
}
```

### 8.2 NVS-backed Repository

```rust
// crates/heizbox-infra/src/persistence/heater_repo.rs

pub struct HeaterSettingsRepository {
    nvs: Arc<Box<dyn NvsDriver>>,
}

impl HeaterSettingsRepository {
    const NAMESPACE: &'static str = "heater";

    pub async fn load(&self) -> Result<HeaterSettings, RepositoryError> {
        Ok(HeaterSettings {
            power: self.nvs.get_u8(Self::NAMESPACE, "power", 100)?,
            mode: self.load_enum("mode", HeatingMode::Preset)?,
            presets: [
                self.nvs.get_u16(Self::NAMESPACE, "preset_1", 185)?,
                self.nvs.get_u16(Self::NAMESPACE, "preset_2", 200)?,
                self.nvs.get_u16(Self::NAMESPACE, "preset_3", 210)?,
                self.nvs.get_u16(Self::NAMESPACE, "preset_4", 220)?,
            ],
            ir_emissivity: self.nvs.get_u8(Self::NAMESPACE, "ir_emissivity", 95)?,
            ir_correction: self.nvs.get_i32(Self::NAMESPACE, "ir_correction", 0)? as i8,
            temp_sensor_read_interval: self.nvs.get_u16(Self::NAMESPACE, "read_interval", 220)?,
            temp_sensor_off_time: self.nvs.get_u16(Self::NAMESPACE, "off_time", 0)?,
        })
    }

    pub async fn save(&self, settings: &HeaterSettings) -> Result<(), RepositoryError> {
        self.nvs.set_u8(Self::NAMESPACE, "power", settings.power)?;
        self.nvs.set_u16(Self::NAMESPACE, "preset_1", settings.presets[0])?;
        self.nvs.set_u16(Self::NAMESPACE, "preset_2", settings.presets[1])?;
        self.nvs.set_u16(Self::NAMESPACE, "preset_3", settings.presets[2])?;
        self.nvs.set_u16(Self::NAMESPACE, "preset_4", settings.presets[3])?;
        self.nvs.set_u8(Self::NAMESPACE, "ir_emissivity", settings.ir_emissivity)?;
        Ok(())
    }

    fn load_enum<T: Default>(
        &self,
        key: &str,
        default: T,
    ) -> Result<T, RepositoryError> {
        // Simplified; real impl would deserialize from string
        Ok(default)
    }
}
```

---

## 9. FEHLERBEHANDLUNG

### Strategie: Domänenspezifische Fehler + Result-basiert

```rust
// crates/heizbox-core/src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Heater error: {0}")]
    Heater(#[from] HeaterError),

    #[error("Sensor error: {0}")]
    Sensor(#[from] SensorError),

    #[error("Calibration failed: {0}")]
    Calibration(String),

    #[error("Configuration invalid: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Error)]
pub enum HeaterError {
    #[error("Temperature cutoff exceeded: {actual}°C > {limit}°C")]
    CutoffExceeded { actual: u16, limit: u16 },

    #[error("Cycle timeout: {duration}ms > {limit}ms")]
    TimeoutExceeded { duration: u32, limit: u32 },

    #[error("Invalid temperature reading")]
    InvalidTemperature,

    #[error("Heater not initialized")]
    NotInitialized,

    #[error("Invalid state transition")]
    InvalidStateTransition,
}

#[derive(Debug, Error)]
pub enum SensorError {
    #[error("I2C communication failed")]
    I2cFailed,

    #[error("Sensor not responding")]
    NoResponse,

    #[error("Calibration data corrupted")]
    CorruptedCalibration,
}

// Infra level errors wrap lower level
#[derive(Debug, Error)]
pub enum InfraError {
    #[error("Persistence error")]
    Persistence(#[from] PersistenceError),

    #[error("Network error")]
    Network(#[from] NetworkError),

    #[error("HAL error")]
    Hal(#[from] HalError),
}

// Never use unwrap() - use Result propagation
pub type DomainResult<T> = Result<T, DomainError>;
pub type InfraResult<T> = Result<T, InfraError>;
```

**Error Handling Patterns:**

```rust
// ✅ Good: Propagate errors
async fn update_heater(sm: &mut HeaterSm<Heating>) -> DomainResult<()> {
    let temp = read_sensor().await?;
    sm = sm.update_temperature(temp)?;
    Ok(())
}

// ❌ Never: Unwrap in production code
async fn update_heater_bad(sm: &mut HeaterSm<Heating>) -> DomainResult<()> {
    let temp = read_sensor().await.unwrap(); // ❌ CRASH on error
    Ok(())
}

// ✅ Recoverable errors: Use expect() only with reason
async fn load_config() -> DomainResult<Config> {
    Config::load().await
        .inspect_err(|e| log::error!("Config load failed: {}", e))
}

// ✅ Convert between error types at boundaries
impl From<HalError> for DomainError {
    fn from(e: HalError) -> Self {
        DomainError::Sensor(SensorError::I2cFailed)
    }
}
```

---

## 10. OTA UPDATE SERVICE

```rust
// crates/heizbox-infra/src/ota/service.rs

pub struct OtaService {
    http: Arc<HttpClient>,
    state: OtaState,
}

#[derive(Clone, Copy)]
enum OtaState {
    Idle,
    Downloading { percent: u8 },
    Installing,
    Failed(OtaError),
}

impl OtaService {
    pub async fn check_and_update(&mut self) -> InfraResult<()> {
        self.state = OtaState::Downloading { percent: 0 };

        let firmware_url = self.get_latest_firmware_url().await?;
        let binary = self.download_firmware(&firmware_url).await?;

        // Verify signature
        self.verify_firmware_signature(&binary)?;

        self.state = OtaState::Installing;

        // Hand off to esp-idf OTA
        esp_idf_svc::ota::EspOta::default()
            .initiate_ota()
            .map_err(|_| InfraError::OtaFailed)?;

        Ok(())
    }

    async fn download_firmware(&mut self, url: &str) -> InfraResult<Vec<u8>> {
        let mut binary = Vec::with_capacity(1024 * 512); // 512 KB max
        let mut response = self.http.get_stream(url).await?;

        let total_size = response.content_length();
        let mut downloaded = 0;

        loop {
            let chunk = response.read_chunk(4096).await?;
            if chunk.is_empty() {
                break;
            }

            binary.extend_from_slice(&chunk);
            downloaded += chunk.len();

            let percent = (downloaded * 100 / total_size) as u8;
            self.state = OtaState::Downloading { percent };

            // Publish progress
            publish_event(DomainEvent::OtaProgress { percent }).await;
        }

        Ok(binary)
    }

    fn verify_firmware_signature(&self, binary: &[u8]) -> InfraResult<()> {
        // Use ESP32 Secure Boot
        esp_idf_svc::ota::verify_signature(binary)
            .map_err(|_| InfraError::OtaFailed)
    }
}
```

---

## 11. RENDERING ARCHITECTURE

### Deklarative Screen State → FrameBuffer

```rust
// crates/heizbox-app/src/screen/render.rs

pub trait RenderEngine {
    fn clear(&mut self, color: Rgb565);
    fn draw_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Rgb565);
    fn draw_text(&mut self, x: u16, y: u16, text: &str, color: Rgb565);
    fn draw_gradient(&mut self, x: u16, y: u16, w: u16, h: u16, from: Rgb565, to: Rgb565);
}

pub struct FrameBufferRenderer {
    fb: FrameBuffer,
    font: EmbeddedGraphics,
}

impl RenderEngine for FrameBufferRenderer {
    fn clear(&mut self, color: Rgb565) {
        self.fb.fill(color);
    }

    fn draw_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Rgb565) {
        // Using embedded-graphics for vector rendering
        Rectangle::new(
            Point::new(x as i32, y as i32),
            Size::new(w as u32, h as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(&mut self.fb)
        .ok();
    }

    fn draw_text(&mut self, x: u16, y: u16, text: &str, color: Rgb565) {
        Text::new(text, Point::new(x as i32, y as i32))
            .into_styled(TextStyle::new(self.font, color))
            .draw(&mut self.fb)
            .ok();
    }
}

// Render Fire Screen
pub async fn render_fire_screen(
    state: &FireScreenState,
    renderer: &mut FrameBufferRenderer,
) -> Result<(), ScreenError> {
    renderer.clear(Rgb565::from((35, 0, 70))); // Dark purple BG

    // Render gradient based on temperature
    let color = get_temp_color(state.heater.current_temp);
    let fill_height = (state.heater.current_temp as u32 * 240) / state.heater.target_temp as u32;
    renderer.draw_gradient(0, (240 - fill_height as u16), 280, fill_height as u16, color, color);

    // Render temperature display
    let temp_str = format!("{}°C", state.heater.current_temp);
    renderer.draw_text(50, 50, &temp_str, Rgb565::from((234, 226, 243)));

    // Render limit
    let limit_str = format!("LIMIT: {}°C", state.heater.target_temp);
    renderer.draw_text(150, 50, &limit_str, Rgb565::from((234, 226, 243)));

    // Render consumption
    let consumption_str = format!("{:.2}g", state.consumption.session);
    renderer.draw_text(10, 210, &consumption_str, Rgb565::from((180, 180, 190)));

    Ok(())
}

fn get_temp_color(temp: u16) -> Rgb565 {
    match temp {
        0..=165 => Rgb565::from((30, 202, 211)),    // Cyan
        166..=180 => Rgb565::from((46, 204, 113)),  // Green
        181..=195 => Rgb565::from((241, 196, 15)),  // Yellow
        196..=215 => Rgb565::from((230, 126, 34)),  // Orange
        _ => Rgb565::from((192, 57, 43)),           // Red
    }
}
```

---

## 12. TESTING STRATEGIE

### Unit Tests (Domain)

```rust
// crates/heizbox-core/tests/heater_sm.rs

#[cfg(test)]
mod heater_sm_tests {
    use heizbox_core::heater::*;

    #[test]
    fn test_idle_to_heating_transition() {
        let config = HeaterConfig::default();
        let sm = HeaterSm::<Idle>::new(config);
        
        // This should compile ✅
        let _heating: HeaterSm<Heating> = sm.start_heating(1000).unwrap();
    }

    #[test]
    fn test_temperature_update_safety_check() {
        let config = HeaterConfig {
            target_temp: 200,
            ..Default::default()
        };
        let sm = HeaterSm::<Idle>::new(config).start_heating(1000).unwrap();

        // Temperature exceeds cutoff → Error
        let result = sm.update_temperature(225, 2000);
        assert!(matches!(result, Err(HeaterError::CutoffExceeded)));
    }

    #[test]
    fn test_heating_to_paused_transition() {
        let config = HeaterConfig::default();
        let sm = HeaterSm::<Idle>::new(config)
            .start_heating(1000)
            .unwrap()
            .update_temperature(150, 2000)
            .unwrap();

        // This should compile ✅
        let _paused: HeaterSm<Paused> = sm.pause();
    }

    #[test]
    fn test_paused_to_heating_transition() {
        let config = Heater