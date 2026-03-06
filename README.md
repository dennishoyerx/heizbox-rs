# Heizbox-RS — Technische Projektdokumentation

> Rust-basierte Firmware für einen ESP32 Induktionsheizer  
> Version 0.1.0 · März 2026 · Workspace: heizbox-rs / 6 Crates

---

## Inhaltsverzeichnis

1. [Projektübersicht](#1-projektübersicht)
2. [Architektur](#2-architektur)
3. [heizbox-core — Domain-Schicht](#3-heizbox-core--domain-schicht)
4. [heizbox-hal — Hardware Abstraction Layer](#4-heizbox-hal--hardware-abstraction-layer)
5. [heizbox-app — Application-Schicht](#5-heizbox-app--application-schicht)
6. [heizbox-infra — Infrastructure-Schicht](#6-heizbox-infra--infrastructure-schicht)
7. [heizbox-esp32 — Einstiegspunkt & HAL-Implementierungen](#7-heizbox-esp32--einstiegspunkt--hal-implementierungen)
8. [System-Integration & Backend-Kommunikation](#8-system-integration--backend-kommunikation)
9. [Testing](#9-testing)
10. [Entwicklungs-Workflow](#10-entwicklungs-workflow)
11. [Bekannte Limitierungen & Nächste Schritte](#11-bekannte-limitierungen--nächste-schritte)
12. [Glossar](#12-glossar)

---

## 1. Projektübersicht

Heizbox-RS ist die vollständige Rust-Reimplementierung der Firmware für einen ESP32-basierten ZVS-Induktionsheizer (DynaVap-kompatibel). Das Projekt ersetzt die ursprüngliche C++-Codebasis durch eine layered, testbare und typsichere Architektur im modernen Rust.

### 1.1 Ziele

- **Typensicherheit** — unmögliche Zustände werden auf Compilerzeit ausgeschlossen (typestate pattern)
- **Schichtenarchitektur** — klare Trennung von Domain, Application, Infrastructure und Hardware
- **Testbarkeit** — Domain- und Application-Logik ist vollständig auf Host-Seite unit-testbar
- **Portabilität** — die Hardware-Abstraktion erlaubt zukünftige Board-Wechsel ohne Domain-Änderungen
- **Wartbarkeit** — kein globaler Zustand, keine unbegründeten `unwrap()`-Aufrufe

### 1.2 Hardware

| Komponente | Beschreibung |
|---|---|
| Mikrocontroller | ESP32 Dev-Board (Xtensa LX6, 240 MHz, 2 Cores, 520 KB SRAM) |
| Display | ST7789 TFT 280×240 px, SPI2 (HSPI) |
| Eingabe | 5-Wege-Joystick direkt an GPIO, Feuer-Taste GPIO 13 |
| Heizelement | ZVS-Induktionsmodul, MOSFET-Gate GPIO 32 |
| IR-Temperatursensor | MLX90614, I²C0 (SCL=27, SDA=26) |
| I²C-Expander | PCF8574, I²C0 (gleicher Bus wie MLX90614) |
| Datenspeicher | NVS (Non-Volatile Storage) im internen Flash |
| Konnektivität | WLAN 802.11 b/g/n (integriert) |

### 1.3 Verzeichnisstruktur

```
heizbox-rs/
├── Cargo.toml               # Workspace-Root
├── .cargo/config.toml       # Build-Target: xtensa-esp32-espidf
└── crates/
    ├── heizbox-core/        # Domain-Schicht (no_std-kompatibel)
    ├── heizbox-hal/         # HAL-Trait-Definitionen
    ├── heizbox-app/         # Application-Schicht (Screens, Navigation)
    ├── heizbox-infra/       # Infrastructure (Netzwerk, NVS, OTA)
    ├── heizbox-esp32/       # ESP32-Einstiegspunkt + HAL-Implementierungen
    └── heizbox-tests/       # Integrationstests (läuft auf Host)
```

---

## 2. Architektur

Das Projekt folgt einer strikten 5-Schichten-Architektur. Abhängigkeiten verlaufen ausschließlich von oben nach unten; zyklische Abhängigkeiten existieren nicht.

| Schicht | Crate | Verantwortung |
|---|---|---|
| Application | `heizbox-app` | Screens, Navigation, Input-Handling |
| Domain | `heizbox-core` | Heater-SM, Events, Modelle, Fehler |
| Infrastructure | `heizbox-infra` | Netzwerk, NVS-Repos, OTA, Clock |
| HAL Traits | `heizbox-hal` | Plattform-unabhängige Treiber-Traits |
| ESP32 Impl. | `heizbox-esp32` | Konkrete Treiber + main.rs |

> ✔ **Invariante:** `heizbox-core` hat keine Abhängigkeit auf andere eigene Crates — es ist der reine Kern.

### 2.1 Abhängigkeitsgraph

```
heizbox-esp32
    ├── heizbox-app
    │       ├── heizbox-core
    │       └── heizbox-hal
    ├── heizbox-infra
    │       ├── heizbox-core
    │       └── heizbox-hal
    ├── heizbox-hal
    │       └── heizbox-core
    └── heizbox-core
```

### 2.2 Concurrency-Modell

Auf dem ESP32 laufen mehrere FreeRTOS-Tasks parallel. Die Kommunikation zwischen ihnen erfolgt (geplant) über FreeRTOS-Queues. Es gibt keinen globalen shared-state außer atomaren Flags.

| Task | Priorität | Core | Aufgabe |
|---|---|---|---|
| `control_task` | 5 | 0 | HeaterSm-Tick, Sensor-Abfrage, Event-Erzeugung |
| `ui_task` | 4 | 0 | Screen-Rendering auf das TFT, ~20 fps |
| `input_task` | 2 | 0 | GPIO-Polling, InputEvent-Dispatch (~50 ms) |
| `network_task` | 3 | 1 | WLAN, WebSocket, Heartbeat, OTA |
| `heartbeat_task` | 2 | 1 | Periodischer Heartbeat an das Backend |

---

## 3. heizbox-core — Domain-Schicht

Enthält das reine Domänenmodell ohne jede Plattformabhängigkeit. Dieser Crate ist eigenständig testbar auf jedem Host.

### 3.1 Heater State Machine (typestate pattern)

Die Heizlogik ist als typestate-Zustandsmaschine implementiert. Der Zustand ist Teil des Typs — ungültige Übergänge werden zur Compilezeit verhindert.

```rust
// Zustandsübergänge sind durch die Typsignatur erzwungen:
let sm = HeaterSm::<Idle>::new(config);
let sm = sm.start_heating(now_ms)?;          // Idle    → Heating
let sm = sm.update_temperature(t, now)?;     // Heating → Heating (mit Safety-Check)
let sm = sm.pause();                         // Heating → Paused
let sm = sm.resume(now_ms);                  // Paused  → Heating
let (sm, result) = sm.finalize();            // Paused  → Idle + CycleResult
```

#### Zustandsübergänge

| Von | Nach | Methode | Seiteneffekte |
|---|---|---|---|
| Idle | Heating | `start_heating(now_ms)` | `cycle_started_at` wird gesetzt |
| Heating | Heating | `update_temperature(t, now)` | Safety-Checks (Cutoff, Timeout) |
| Heating | Paused | `pause()` | `elapsed_ms` wird eingefroren |
| Paused | Heating | `resume(now_ms)` | `cycle_started_at` wird angepasst |
| Paused | Idle | `finalize()` | `CycleResult` wird zurückgegeben |

#### Safety-Checks in `HeaterSm<Heating>`

- **Cutoff-Check:** Wenn `current_temp > target_temp + 20°C` → `CutoffTemperatureExceeded`
- **Sonderfall:** `target_temp == 420` deaktiviert den Cutoff-Check
- **Timeout-Check:** Wenn `cycle_duration_ms > auto_stop_time_ms` → `CycleTimeoutExceeded`

### 3.2 Domain Events

Alle signifikanten Zustandsänderungen erzeugen einen `DomainEvent`. Events sind serialisierbar (Serde) und werden über WebSocket an das Backend übertragen.

| Event | Payload | Auslöser |
|---|---|---|
| `HeatingStarted` | `target_temp`, `timestamp_ms` | `HeaterSm::start_heating()` |
| `HeatingPaused` | `current_temp`, `duration_ms` | `HeaterSm::pause()` |
| `CycleFinished` | `CycleResult` (duration, max_temp) | `HeaterSm::finalize()` |
| `TemperatureUpdated` | `current`, `ambient`, `raw_ir` | Sensor-Task alle ~220 ms |
| `WifiConnected` | `ssid` | `WifiDriver::connect()` |
| `WifiDisconnected` | `reason` | WLAN-Verbindungsverlust |
| `WebSocketConnected` | — | `WebSocketClient` |
| `OtaProgress` | `percent` (0–100) | `OtaService::download()` |
| `OtaCompleted` | — | `OtaService::apply()` |

### 3.3 Fehlertypen

Alle Fehler sind domänenspezifisch typisiert. Es gibt keine String-Fehlermeldungen im Kernpfad — nur strukturierte Enums, die vollständig gematchet werden können.

| Typ | Varianten |
|---|---|
| `HeaterError` | `CutoffTemperatureExceeded`, `CycleTimeoutExceeded`, `InvalidTemperatureReading` |
| `SensorError` | `I2cFailed`, `SpiFailed`, `NotInitialized`, `InvalidCalibration`, `NoResponse` |
| `DomainError` | `Heater(HeaterError)`, `Sensor(SensorError)`, `Calibration`, `InvalidStateTransition` |
| `NetworkError` | `TlsError`, `HttpError(u16)`, `DnsError`, `Timeout`, `NotConnected`, `ReconnectFailed` |
| `NvsError` | `KeyNotFound`, `TypeMismatch`, `NvsFull`, `InvalidValue`, `Uninitialized` |
| `PersistenceError` | `NvsError`, `DeserializationError(String)` |
| `HalError` | `Nvs`, `Gpio`, `I2c`, `Spi`, `Adc`, `Timer`, `Wifi` (je mit Inner-Error) |
| `InfraError` | `Persistence`, `Network`, `Hal` (From-Konvertierungen vorhanden) |

### 3.4 Consumption-Modell

`ConsumptionData` aggregiert die Gerätelebensdauer-Statistiken und wird über NVS dauerhaft gespeichert.

```rust
pub struct ConsumptionData {
    pub total_cycles:      u32,  // Anzahl abgeschlossener Heizzyklen
    pub total_duration_ms: u64,  // Kumulierte Heizdauer in ms
}

impl ConsumptionData {
    pub fn record_cycle(&mut self, duration_ms: u32) { ... }
    pub fn total_duration_secs(&self) -> u64 { ... }
}
```

---

## 4. heizbox-hal — Hardware Abstraction Layer

Definiert plattformunabhängige Traits für alle Hardware-Peripherie. Konkrete Implementierungen befinden sich ausschließlich in `heizbox-esp32`.

### 4.1 Trait-Übersicht

| Trait | Methoden | Async? | Bemerkung |
|---|---|---|---|
| `GpioDriver` | `set_output`, `set_input`, `write`, `read` | Nein | `Send+Sync` |
| `I2cDriver` | `write`, `read`, `write_read` | Ja | `async-trait` |
| `SpiDriver` | `write`, `read`, `transfer` | Nein | Sync |
| `NvsDriver` | `get_*/set_*/erase` (alle Typen: u8..f32, str) | Nein | `Send+Sync` |
| `AdcDriver` | `read(pin) → u16` | Nein | Sync |
| `TimerDriver` | `start`, `stop`, `is_running` | Nein | Sync |
| `WifiDriver` | `connect`, `disconnect`, `is_connected`, `get_ip` | Ja | `async-trait` |

### 4.2 Pin-Belegung

> ⚠ **Mehrere Pin-Konflikte erfordern Hardware-Review** — siehe Tabelle unten.

| Funktion | GPIO | Konflikt |
|---|---|---|
| Display MOSI (ST7789) | 23 | — |
| Display SCK | 18 | Geteilt mit Thermocouple SCK |
| Display CS | 5 | — |
| Display DC | 4 | — |
| Display RST | 15 | — |
| Display Backlight (BL) | 16 | — |
| Joystick UP | 1 | — |
| Joystick DOWN | 0 | — |
| Joystick LEFT | 3 | — |
| Joystick RIGHT | 2 | ⚠ STATUS_LED (Pin 2) |
| Joystick PRESS | 4 | — |
| Feuer-Taste | 13 | — |
| Heater MOSFET Gate | 32 | — |
| I²C SCL (PCF8574/MLX) | 27 | — |
| I²C SDA (PCF8574/MLX) | 26 | — |
| PCF8574 INT | 25 | ⚠ SPEAKER (Pin 25) |
| Status LED | 2 | ⚠ Joystick RIGHT (Pin 2) |
| Speaker / Buzzer | 25 | ⚠ PCF8574 INT (Pin 25) |

### 4.3 IpAddr

Um externe Abhängigkeiten in `heizbox-hal` minimal zu halten, wird ein eigenes `IpAddr`-Struct definiert statt `std::net::IpAddr`.

```rust
pub struct IpAddr(pub [u8; 4]);
// Display-Impl: "192.168.1.42"
```

---

## 5. heizbox-app — Application-Schicht

Verwaltet die gesamte Benutzerschnittstelle: Screen-Lifecycle, Navigation, Input-Verarbeitung und Event-Dispatch. Kein Hardware-Zugriff — ausschließlich über Trait-Objekte.

### 5.1 Screen-Trait

Alle Bildschirme implementieren den `Screen`-Trait. Dies ermöglicht eine einheitliche Verarbeitung im `ScreenManager`.

```rust
#[async_trait]
pub trait Screen: Send {
    async fn on_enter(&mut self);
    async fn on_exit(&mut self);
    async fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError>;
    async fn update(&mut self, event: DomainEvent)     -> Result<(), ScreenError>;
    async fn render(&self)                             -> Result<FrameBuffer, ScreenError>;
}
```

### 5.2 Screens

| Screen | Beschreibung | Navigation-Ziele |
|---|---|---|
| `StartupScreen` | Boot-Splashscreen, wechselt sofort zu Fire | → Fire |
| `FireScreen` | Aktiver Heizbildschirm, Temperaturanzeige, Toggle | → Menu |
| `MenuScreen` | Einstellungsmenü, scrollbar per Joystick Up/Down | → Fire |
| `Screensaver` | Einbrenn-Schutz nach Idle-Timeout | → Fire (Wakeup) |
| `OtaUpdateScreen` | OTA-Fortschrittsanzeige | → Fire (nach Reboot) |

### 5.3 Navigation FSM

Die `NavigationFsm` validiert alle Zustandsübergänge. Ungültige Navigationen werden mit `NavError` zurückgewiesen und nicht stillschweigend ignoriert.

```rust
pub struct NavigationFsm {
    current: ScreenType,
    history: heapless::Vec<ScreenType, 8>,  // max. 8 Ebenen Rücknavigation
}

impl NavigationFsm {
    pub fn navigate_to(&mut self, target: ScreenType) -> Result<(), NavError>;
    pub fn navigate_back(&mut self)                   -> Result<(), NavError>;
    pub fn current(&self)                             -> ScreenType;
}
```

### 5.4 InputHandler

Der `InputHandler` verarbeitet GPIO-Rohdaten (pressed/released) und klassifiziert sie als `Press`, `LongPress` oder `Release`. Intern nutzt er eine kleine State Machine mit drei Zuständen.

| Zustand | Übergang | Ausgabe |
|---|---|---|
| Normal | Taste gedrückt > 300 ms → LongPressActive | LongPress-Event |
| Normal | Taste < 300 ms gehalten und losgelassen | Press-Event |
| LongPressActive | Taste losgelassen | Release-Event |
| MenuMode | Up/Down → Press; Left/Right → Release + Normal | Press/Release |

### 5.5 FrameBuffer

Der `FrameBuffer` hält die gerenderten Pixeldaten (RGB565, 16 bit/Pixel) für einen Frame. Für 280×240 px beträgt die Größe 134.400 Bytes und wird per `heapless::Vec` ohne Heap-Allokation verwaltet.

```rust
pub struct FrameBuffer {
    pub data:   heapless::Vec<u8, 134400>,
    pub width:  u16,  // 280
    pub height: u16,  // 240
}
```

---

## 6. heizbox-infra — Infrastructure-Schicht

Implementiert alle Dienste, die externe Ressourcen benötigen: NVS-Repositories, HTTP/WebSocket-Client, OTA-Service und NTP-Uhrsynchronisation.

### 6.1 NVS Repositories

Alle Persistierungsoperationen erfolgen ausschließlich über typisierte Repositories. Direkter NVS-Zugriff aus der Domain ist nicht erlaubt.

| Repository | Struct | Speichert |
|---|---|---|
| `HeaterConfigRepository` | `HeaterConfigRepository<N: NvsDriver>` | `power`, `target_temp`, `auto_stop_time_ms` |
| `HeaterSettingsRepository` | `HeaterSettingsRepository<N: NvsDriver>` | Alle Benutzereinstellungen + Presets |

```rust
let repo = HeaterConfigRepository::new(nvs);
let config: HeaterConfig = repo.load()?;  // liest aus NVS, gibt Default zurück falls nicht vorhanden
repo.save(&config)?;                      // schreibt in NVS
```

### 6.2 WebSocket-Client

Der `WebSocketClient` verwaltet eine persistente Verbindung zum Cloudflare-Backend. Bei Verbindungsabbruch greift ein exponentieller Backoff-Mechanismus mit bis zu 10 Versuchen.

```rust
pub struct WebSocketClient {
    url:     String,
    socket:  Option<WsHandle>,
    state:   WsState,            // Disconnected | Connecting | Connected
    backoff: ExponentialBackoff, // 100ms → 200ms → 400ms → ... → 30s
}

// Verwendung:
ws_client.send_event(&DomainEvent::HeatingStarted { ... }).await?;
```

> ℹ Der `ExponentialBackoff` startet bei 100 ms und verdoppelt sich bei jedem Fehlversuch bis maximal 30 Sekunden.

### 6.3 HTTP-Client

`HttpClient` kapselt alle REST-Aufrufe zum Backend. Er abstrahiert das TLS-Handling und stellt typsichere Methoden bereit.

```rust
let client = HttpClient::new("https://backend.hzbx.de");
client.post_json("/api/heat_cycles", &json_body).await?;
client.get("/api/firmware/latest").await?;
```

### 6.4 OTA-Service

`OtaService` lädt Firmware-Updates vom Backend herunter und übergibt sie an die ESP-IDF OTA-Partition. Der Download-Fortschritt wird als `DomainEvent` veröffentlicht.

| Zustand | Beschreibung |
|---|---|
| `Idle` | Kein Update aktiv |
| `Downloading { percent: u8 }` | Download läuft, 0–100 % |
| `Installing` | Firmware wird in OTA-Partition geschrieben |

### 6.5 ClockManager

`ClockManager` verwaltet den UNIX-Zeitstempel und die NTP-Synchronisation. Nach dem ersten NTP-Sync wird ein Offset gespeichert, der zu jedem `esp_timer_get_time()`-Wert addiert wird.

```rust
let mut clock = ClockManager::new();
clock.set_offset(ntp_unix_timestamp);
let now: i64   = clock.now_unix();
let synced: bool = clock.is_synced();
```

---

## 7. heizbox-esp32 — Einstiegspunkt & HAL-Implementierungen

Dieser Crate ist der einzige, der `esp-idf-hal` und `esp-idf-svc` direkt importiert. Er enthält `main.rs` sowie alle konkreten Treiberimplementierungen in `hal_impl/`.

### 7.1 main.rs

`main()` initialisiert alle HAL-Treiber, erzeugt die `DeviceApp` und spawnt die FreeRTOS-Tasks als `std::thread`.

```rust
fn main() -> anyhow::Result<()> {
    link_patches();                   // esp-idf-svc Pflicht
    EspLogger::initialize_default();

    let nvs  = NvsImpl::new()?;
    let gpio = GpioImpl::new();
    let i2c  = I2cImpl::new();
    // ...

    thread::Builder::new().name("control".into())
        .stack_size(8 * 1024).spawn(control_task)?;
    // ... weitere Tasks

    loop { thread::sleep(Duration::from_secs(5)); }
}
```

### 7.2 HAL-Implementierungen

Alle Treiber-Stubs sind so strukturiert, dass sie 1:1 durch echte ESP-IDF-Calls ersetzt werden können.

| Datei | Struct | Produktions-Ersatz |
|---|---|---|
| `gpio_impl.rs` | `GpioImpl` | `esp_idf_hal::gpio::PinDriver` |
| `i2c_impl.rs` | `I2cImpl` | `esp_idf_hal::i2c::I2cDriver` |
| `spi_impl.rs` | `SpiImpl` | `esp_idf_hal::spi::SpiDeviceDriver` |
| `nvs_impl.rs` | `NvsImpl` | `esp_idf_svc::nvs::EspNvs` |
| `wifi_impl.rs` | `WifiImpl` | `esp_idf_svc::wifi::EspWifi` |
| `adc_impl.rs` | `AdcImpl` | `esp_idf_hal::adc::AdcDriver` |
| `timer_impl.rs` | `TimerImpl` | `esp_idf_hal::timer::TimerDriver` |

### 7.3 Konfiguration (config.rs)

| Konstante | Wert | Bedeutung |
|---|---|---|
| `DEVICE_ID` | `"heizbox-01"` | WebSocket-Client-ID |
| `BACKEND_WS_URL` | `wss://backend.hzbx.de/ws/status` | WebSocket-Endpunkt |
| `BACKEND_HTTP_URL` | `https://backend.hzbx.de` | REST-Endpunkt |
| `NTP_SERVER` | `pool.ntp.org` | NTP-Quelle |
| `HEARTBEAT_INTERVAL_MS` | `5000` | Heartbeat-Takt |

### 7.4 Build-Konfiguration (.cargo/config.toml)

```toml
[build]
target = "xtensa-esp32-espidf"

[target.xtensa-esp32-espidf]
linker = "ldproxy"
runner = "espflash flash --monitor"
rustflags = ["--cfg", "espidf_time64"]

[env]
MCU             = "esp32"
ESP_IDF_VERSION = "v5.3.3"
```

---

## 8. System-Integration & Backend-Kommunikation

Das Gesamtsystem besteht aus drei Komponenten: ESP32-Firmware (dieser Crate), einem Cloudflare-Backend (TypeScript/Hono) und einem React-Frontend. Die Kommunikation läuft ausschließlich über das Cloudflare-Backend.

### 8.1 REST-API-Endpunkte

| Methode | Pfad | Sender | Beschreibung |
|---|---|---|---|
| `POST` | `/api/heartbeat` | ESP32 | Lebenszeichen; leitet an DeviceStatus-DO weiter |
| `GET` | `/api/heat_cycles` | Frontend | Heizzyklen des aktuellen Tages |
| `GET` | `/api/heat_cycles/create` | ESP32 | Legacy: Zyklus via Query-String anlegen |
| `GET` | `/api/statistics` | Frontend | Aggregierte Statistiken (`range`-Parameter) |
| `GET` | `/api/session` | Frontend | Daten der laufenden Heiz-Session |
| `GET` | `/api/device-status/:id/status` | Frontend | Echtzeit-Status direkt vom Durable Object |

### 8.2 WebSocket-Protokoll

Die WebSocket-Verbindung läuft gegen den Cloudflare Worker-Endpunkt. Alle Nachrichten sind JSON-Objekte mit einem `type`-Feld.

```
// Verbindungs-URL:
wss://<worker-url>/ws/status?deviceId=<id>&type=device

// Vom ESP32 an das Backend:
{ "type": "statusUpdate",       "isOn": true, "isHeating": true }
{ "type": "heartbeat" }
{ "type": "heatCycleCompleted", "duration": 45000, "cycle": 3 }
{ "type": "sessionUpdate",      "clicks": 5, "lastClick": 1711234567 }

// Vom Backend an das Frontend (Broadcast):
{ "type": "statusUpdate",  "isOn": boolean, "isHeating": boolean }
{ "type": "sessionCreated" }
{ "type": "sessionData",   "clicks": 5, "sessionStart": ... }
```

### 8.3 Datenbank-Schema (Cloudflare D1)

#### Tabelle: `heat_cycles`

| Spalte | Typ | Beschreibung |
|---|---|---|
| `id` | `TEXT` | Primärschlüssel (UUID) |
| `device_id` | `TEXT` | Fremdschlüssel → `devices.id` |
| `start_time` | `INTEGER` | UNIX-Timestamp UTC (Beginn) |
| `end_time` | `INTEGER` | UNIX-Timestamp UTC (Ende) |
| `duration_ms` | `INTEGER` | Heizdauer in Millisekunden |
| `target_temperature` | `INTEGER` | Zieltemperatur in °C |
| `max_temperature` | `INTEGER` | Höchste gemessene Temperatur |
| `voltage_start` | `REAL` | Batteriespannung zu Beginn (V) |
| `voltage_end` | `REAL` | Batteriespannung am Ende (V) |
| `created_at` | `INTEGER` | Erstellungszeitpunkt |

#### Tabelle: `devices`

| Spalte | Typ | Beschreibung |
|---|---|---|
| `id` | `TEXT` | ESP32 Chip-ID |
| `name` | `TEXT` | Benutzerdefinierter Gerätename |
| `firmware_version` | `TEXT` | Aktuell installierte Firmware-Version |
| `last_seen` | `INTEGER` | Letzter Heartbeat (UNIX-Timestamp) |

---

## 9. Testing

`heizbox-tests` enthält alle host-seitigen Integrationstests. Sie laufen ohne ESP32-Hardware mit `cargo test` auf dem Entwicklungsrechner.

### 9.1 Ausführung

```bash
# Alle Tests ausführen:
cargo test -p heizbox-tests

# Einzelnen Test ausführen:
cargo test -p heizbox-tests heater_sm_tests::cutoff_exceeded

# Mit Ausgabe:
cargo test -p heizbox-tests -- --nocapture
```

### 9.2 Testfälle (heater_sm.rs)

| Testname | Prüft |
|---|---|
| `idle_to_heating` | Übergang Idle → Heating, `cycle_started_at` korrekt |
| `cutoff_exceeded_returns_error` | 225°C > 220°C (200+20) → `CutoffTemperatureExceeded` |
| `timeout_exceeded_returns_error` | 91 000 ms > 90 000 ms → `CycleTimeoutExceeded` |
| `normal_update_succeeds` | Temperatur-Update bei 150°C, `cycle_duration_ms` korrekt |
| `heating_to_paused` | Übergang Heating → Paused kompiliert und läuft |
| `paused_to_heating_resume` | `resume()` passt `cycle_started_at` korrekt an |
| `paused_to_idle_finalize` | `CycleResult.max_temp`, `duration_ms`, `started_at` korrekt |
| `target_reached_flag` | `is_target_reached()` bei 200°C == 200°C → `true` |

### 9.3 Testfälle (persistence.rs)

| Testname | Prüft |
|---|---|
| `heater_config_defaults` | `HeaterConfig::with_defaults()` liefert korrekte Standardwerte |

### 9.4 Testfälle (network.rs)

| Testname | Prüft |
|---|---|
| `network_error_display` | `NetworkError::Timeout` hat nicht-leere Display-Ausgabe |
| `reconnect_failed_display` | `NetworkError::ReconnectFailed` enthält `'retries'` im Text |

---

## 10. Entwicklungs-Workflow

### 10.1 Voraussetzungen

- Rust (stable + nightly für xtensa): `rustup toolchain install nightly`
- espup: `cargo install espup && espup install`
- espflash: `cargo install espflash`
- ldproxy: `cargo install ldproxy`
- ESP-IDF v5.3.3 (wird automatisch durch `embuild` heruntergeladen)

### 10.2 Build

```bash
# Alle host-seitigen Crates prüfen (ohne ESP32-Target):
cargo check -p heizbox-core -p heizbox-app -p heizbox-tests

# Tests ausführen:
cargo test -p heizbox-tests

# ESP32-Firmware bauen:
cargo build -p heizbox-esp32 --release

# Flashen und Monitor:
cargo run -p heizbox-esp32 --release
```

### 10.3 Empfohlene Entwicklungsreihenfolge

- **Domain-Änderungen:** zuerst in `heizbox-core`, dann Tests in `heizbox-tests`
- **Neue HAL-Peripherie:** Trait in `heizbox-hal` definieren, Stub in `heizbox-esp32` eintragen
- **Neuer Screen:** `FireScreen`/`MenuScreen` als Vorlage, `Screen`-Trait implementieren
- **Netzwerkänderungen:** `ws_client.rs` / `http_client.rs` ohne Hardware testbar über Mock-Stubs

---

## 11. Bekannte Limitierungen & Nächste Schritte

### 11.1 Aktuelle Stubs (noch nicht produktionsreif)

| Komponente | Status | Was fehlt |
|---|---|---|
| `GpioImpl` | Stub (HashMap) | Echte `esp_idf_hal::gpio::PinDriver`-Calls |
| `I2cImpl` | Stub (no-op) | `esp_idf_hal::i2c::I2cDriver` + MLX90614-Treiber |
| `SpiImpl` | Stub (no-op) | `esp_idf_hal::spi::SpiDeviceDriver` + Display-Init |
| `NvsImpl` | Stub (no-op) | `esp_idf_svc::nvs::EspNvs` mit echter Persistenz |
| `WifiImpl` | Stub | `esp_idf_svc::wifi::EspWifi` + Credentials aus NVS |
| `WebSocketClient` | Stub | `esp_idf_svc::ws::client::EspWebSocketClient` |
| `HttpClient` | Stub (log-only) | `esp_idf_svc::http::client::EspHttpConnection` |
| `EventBus` | no-op | FreeRTOS-Queue oder embassy Channel |
| `FireScreen render` | no-op | Echtes RGB565-Rendering mit `mipidsi` |
| OTA | Stub | `esp_idf_svc::ota::EspOta` + Signaturprüfung |

### 11.2 Pin-Konflikte (Hardware-Review nötig)

- **GPIO 2:** Joystick RIGHT vs. Status LED — nur eine Funktion kann gleichzeitig aktiv sein
- **GPIO 25:** PCF8574 INT vs. Speaker — Konflikt lösen oder Software-Multiplexing
- **GPIO 18:** SCK wird von Display und Thermocouple geteilt — CS-Management erforderlich
- **GPIO 32:** MOSFET Gate und Thermocouple CS potenziell konfliktiv — Pins klären

### 11.3 Geplante Features

- Temperatur-Kalibrierung via MLX90614 mit Emissivitätskorrektur
- Preset-System: 4 Temperaturstufen (Flavour / Balanced / Extraction / Full)
- Rotary-Encoder-Unterstützung für stufenlose Temperaturauswahl
- Screensaver mit konfigurierbarem Timeout
- OTA über WLAN mit Firmware-Signaturprüfung (ESP32 Secure Boot)
- NTP-Synchronisation beim Boot
- Battery-Voltage-Monitoring über ADC

---

## 12. Glossar

| Begriff | Erklärung |
|---|---|
| **Typestate Pattern** | Rust-Muster, bei dem der Zustand eines Objekts durch den Typ kodiert wird — unmögliche Übergänge verursachen Compile-Fehler |
| **ESP-IDF** | Espressif IoT Development Framework — das native SDK für ESP32 |
| **HAL** | Hardware Abstraction Layer — definiert einheitliche Interfaces für Treiber |
| **NVS** | Non-Volatile Storage — schlüssel-/wertbasierter Flash-Speicher des ESP32 |
| **FreeRTOS** | Echtzeit-Betriebssystem, das vom ESP-IDF-Stack transparent verwendet wird |
| **ZVS** | Zero Voltage Switching — Schaltprinzip des Induktionsheizelements |
| **Durable Object** | Cloudflare-Runtime für stateful Edge-Computing mit persistentem Speicher |
| **D1** | Cloudflare's serverlose SQLite-kompatible Datenbank (SQL-über-HTTP) |
| **RGB565** | 16-bit Farbformat: 5 Bit Rot, 6 Bit Grün, 5 Bit Blau |
| **SPI2 / HSPI** | ESP32 SPI-Bus 2 — genutzt für Display (ST7789) und Thermocouple |
| **I²C0** | ESP32 I²C-Bus 0 (SCL=27, SDA=26) — genutzt für MLX90614 und PCF8574 |
| **OTA** | Over-The-Air — Firmware-Update per WLAN ohne physische Verbindung |
| **MLX90614** | Berührungsloser IR-Temperatursensor von Melexis, I²C-Interface |
| **PCF8574** | I²C-GPIO-Expander (8 Bit) — erweitert die verfügbaren GPIO-Pins |

---

*Heizbox-RS · Version 0.1.0 · März 2026*
