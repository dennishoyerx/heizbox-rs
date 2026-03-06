# heizbox-rs — Ticket-Backlog

> Hierarchie: **Crate → Epic → Aufgabe**  
> Prioritäten: 🔴 kritisch · 🟠 hoch · 🟡 mittel · 🟢 niedrig  
> Status: ⬜ offen · 🔄 in Arbeit · ✅ erledigt

---

## Inhaltsverzeichnis

- [CORE — heizbox-core](#core--heizbox-core)
- [HAL — heizbox-hal](#hal--heizbox-hal)
- [APP — heizbox-app](#app--heizbox-app)
- [INFRA — heizbox-infra](#infra--heizbox-infra)
- [ESP32 — heizbox-esp32](#esp32--heizbox-esp32)
- [TESTS — heizbox-tests](#tests--heizbox-tests)

---

## CORE — heizbox-core

### Epic CORE-E1 · Heater State Machine — Produktionsreife
> Die Typestate-SM ist strukturell vollständig, benötigt aber fehlende Grenzfälle, Kalibrierlogik und erweiterte Statistiken.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| CORE-T1 | `HeaterConfig::with_defaults()` mit realen Gerätewerten befüllen (target_temp, auto_stop_time_ms, power_level) | 🔴 kritisch | ⬜ | `HeaterConfig`, `DEFAULT_TARGET_TEMP`, `config.rs` |
| CORE-T2 | `CycleResult` um Felder `voltage_start: f32`, `voltage_end: f32` und `max_temp: u16` erweitern | 🟠 hoch | ⬜ | `CycleResult`, `heat_cycles`-Tabelle |
| CORE-T3 | Cutoff-Sonderfall `target_temp == 420` (CUTOFF_DISABLED) in eigenem benannten Const dokumentieren und testen | 🟡 mittel | ⬜ | `CUTOFF_DISABLED = 420`, `HeaterSm<Heating>`, `HeaterError` |
| CORE-T4 | `HeatingMode`-Enum (Flavour / Balanced / Extraction / Full) als Domain-Typ in `heizbox-core` definieren — nicht in infra | 🟡 mittel | ⬜ | `HeatingMode`, `HeaterConfig` |
| CORE-T5 | Preset-Mapping `HeatingMode → (target_temp, power_level)` als `const`-Array implementieren | 🟢 niedrig | ⬜ | `HeatingMode`, `HeaterConfig`, `CORE-T4` |
| CORE-T6 | `ConsumptionData::record_cycle()` atomar machen (kein doppeltes Zählen bei Retry) | 🟠 hoch | ⬜ | `ConsumptionData`, `StatsManager` |

---

### Epic CORE-E2 · Domain Events — Vollständigkeit & Serialisierung
> Events müssen alle Szenarien abdecken und zuverlässig über WebSocket übertragen werden.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| CORE-T7 | `DomainEvent::SessionUpdate { clicks, last_click, session_start }` ergänzen (fehlt noch) | 🔴 kritisch | ⬜ | `DomainEvent`, WebSocket-Protokoll `sessionUpdate` |
| CORE-T8 | `DomainEvent::HeartbeatSent` ergänzen für interne Traceability | 🟢 niedrig | ⬜ | `DomainEvent`, `HeartbeatManager` |
| CORE-T9 | Serde-Rename-Attribute für JSON-Feld-Konvention (`camelCase`) auf allen Event-Varianten sicherstellen | 🟠 hoch | ⬜ | `DomainEvent`, `serde(rename_all = "camelCase")` |
| CORE-T10 | `DomainEvent`-Deserialisierung implementieren (aktuell nur Serialize) für eingehende Backend-Nachrichten | 🟡 mittel | ⬜ | `DomainEvent`, `WebSocketClient`, `Deserialize` |

---

### Epic CORE-E3 · Fehlerbehandlung — Vollständigkeit
> Alle Fehlerpfade müssen strukturiert typisiert und ohne `unwrap()` erreichbar sein.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| CORE-T11 | `From<SensorError> for DomainError` implementieren | 🔴 kritisch | ⬜ | `SensorError`, `DomainError` |
| CORE-T12 | `From<NvsError> for PersistenceError` vollständig implementieren (alle Varianten prüfen) | 🟠 hoch | ⬜ | `NvsError`, `PersistenceError` |
| CORE-T13 | Alle `unwrap()` / `expect()` aus heizbox-core entfernen, durch `?`-Operator oder explizites Matching ersetzen | 🔴 kritisch | ⬜ | `DomainError`, `HeaterError`, `SensorError` |

---

## HAL — heizbox-hal

### Epic HAL-E1 · Trait-Vollständigkeit
> Alle benötigten Peripherie-Traits sind definiert; fehlende Fähigkeiten und Rückgabetypen vervollständigen.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| HAL-T1 | `GpioDriver::read_analog(pin) → u16` als separater Trait oder Methode in `AdcDriver` klarstellen — Überschneidung auflösen | 🟡 mittel | ⬜ | `GpioDriver`, `AdcDriver` |
| HAL-T2 | `NvsDriver`-Trait: Rückgabetyp für fehlende Keys explizit auf `Option<T>` statt `Result<T, NvsError::KeyNotFound>` vereinheitlichen | 🟠 hoch | ⬜ | `NvsDriver`, `NvsError` |
| HAL-T3 | `WifiDriver::scan() → Vec<Ssid>` für zukünftiges WLAN-Einstellungs-Screen hinzufügen | 🟢 niedrig | ⬜ | `WifiDriver`, `IpAddr` |
| HAL-T4 | `TimerDriver::elapsed_ms() → u64` als Methode ergänzen (wird von `HeaterSm` benötigt) | 🔴 kritisch | ⬜ | `TimerDriver`, `HeaterSm<Heating>` |

---

## APP — heizbox-app

### Epic APP-E1 · Screen-Rendering implementieren
> Alle `render()`-Implementierungen sind aktuell no-ops. Reales RGB565-Rendering auf dem ST7789 ist das größte offene Feature-Paket.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| APP-T1 | `FrameBuffer`-Rendering-Pipeline aufsetzen: `mipidsi`-Crate oder direktes SPI einbinden, Flush-Mechanismus implementieren | 🔴 kritisch | ⬜ | `FrameBuffer`, `SpiDriver`, `DisplayManager` |
| APP-T2 | `FireScreen::render()`: Temperatur-Anzeige (groß, zentriert), Heiz-Indikator (animiert), Fortschrittsbalken für Zieltemperatur | 🔴 kritisch | ⬜ | `FireScreen`, `FrameBuffer`, `DomainEvent::TemperatureUpdated` |
| APP-T3 | `MenuScreen::render()`: scrollbare Liste mit Joystick-Cursor, aktuell ausgewählter Eintrag hervorheben | 🟠 hoch | ⬜ | `MenuScreen`, `FrameBuffer`, `InputEvent` |
| APP-T4 | `StartupScreen::render()`: Logo/Splashscreen für ~2 Sekunden, danach automatisch zu `FireScreen` | 🟡 mittel | ⬜ | `StartupScreen`, `Navigation`, `FrameBuffer` |
| APP-T5 | `ScreensaverScreen` implementieren: Timeout nach konfigurierbarer Idle-Zeit, Wakeup durch beliebige Taste | 🟡 mittel | ⬜ | `ScreensaverScreen`, `Screen`, `InputEvent`, `NavigationFsm` |
| APP-T6 | `OtaUpdateScreen::render()`: Fortschrittsbalken für OTA-Download-Prozent | 🟢 niedrig | ⬜ | `OtaUpdateScreen`, `DomainEvent::OtaProgress`, `FrameBuffer` |
| APP-T7 | `StatusBar` auf allen Screens dauerhaft einblenden: WLAN-Icon, Uhrzeit, Batteriestand | 🟠 hoch | ⬜ | `StatusBar`, `FrameBuffer`, `WifiDriver`, `ClockManager` |

---

### Epic APP-E2 · Input-Handling & Navigation
> Grundstruktur ist vorhanden, aber konkrete Button-Aktionen pro Screen und Navigationsübergänge fehlen.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| APP-T8 | `FireScreen::handle_input()`: Center-Druck → Heizen starten/stoppen, Up/Down → Zieltemperatur ändern | 🔴 kritisch | ⬜ | `FireScreen`, `InputEvent`, `DomainEvent::HeatingStarted` |
| APP-T9 | `MenuScreen::handle_input()`: Up/Down → scrollen, Center → Auswahl bestätigen, Left → zurück zu Fire | 🟠 hoch | ⬜ | `MenuScreen`, `InputEvent`, `Navigation`, `NavigationFsm` |
| APP-T10 | Long-Press auf Feuer-Taste (GPIO 13): Sofortabbruch des Heizvorgangs unabhängig vom Screen | 🟠 hoch | ⬜ | `InputHandler`, `InputEvent::LongPress`, `HeaterSm`, `FIRE_BTN_PIN` |
| APP-T11 | `EventBus` mit FreeRTOS-Queue oder `heapless::spsc::Queue` implementieren (aktuell no-op) | 🔴 kritisch | ⬜ | `EventBus`, `DomainEvent`, `DeviceApp` |

---

### Epic APP-E3 · DeviceApp-Orchestrierung
> `DeviceApp` koordiniert alle Manager-Klassen; der Hauptloop und Event-Routing fehlen noch.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| APP-T12 | `DeviceApp::run_control_tick()`: Sensor lesen → `HeaterSm::update_temperature()` → `DomainEvent` erzeugen → `EventBus::publish()` | 🔴 kritisch | ⬜ | `DeviceApp`, `HeaterSm`, `DomainEvent`, `EventBus`, `I2cDriver` |
| APP-T13 | `DeviceApp::run_ui_tick()`: `ScreenManager::render()` aufrufen → `FrameBuffer` via SPI flushen | 🔴 kritisch | ⬜ | `DeviceApp`, `ScreenManager`, `FrameBuffer`, `SpiDriver` |
| APP-T14 | Screensaver-Timeout-Logik in `DeviceApp` integrieren: Idle-Timer auf jede `InputEvent`-Aktivität zurücksetzen | 🟡 mittel | ⬜ | `DeviceApp`, `ScreensaverScreen`, `NavigationFsm`, `TimerDriver` |

---

## INFRA — heizbox-infra

### Epic INFRA-E1 · WebSocket-Client — Produktionsimplementierung
> Aktuell Stub; muss gegen `esp_idf_svc::ws::client::EspWebSocketClient` ausgetauscht werden.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| INFRA-T1 | `WebSocketClient::connect()` mit `EspWebSocketClient` implementieren; `deviceId` und `type=device` als Query-Parameter übergeben | 🔴 kritisch | ⬜ | `WebSocketClient`, `WsState`, `BACKEND_WS_URL`, `DEVICE_ID` |
| INFRA-T2 | `WebSocketClient::send_event()` mit echter JSON-Serialisierung und Frame-Send implementieren | 🔴 kritisch | ⬜ | `WebSocketClient`, `DomainEvent`, `serde_json` |
| INFRA-T3 | `WebSocketClient` eingehende Nachrichten empfangen und als `DomainEvent` deserialisieren | 🟠 hoch | ⬜ | `WebSocketClient`, `DomainEvent`, `EventBus`, `CORE-T10` |
| INFRA-T4 | Exponential-Backoff-Reconnect in den echten `EspWebSocketClient`-Lifecycle einbauen | 🟠 hoch | ⬜ | `ExponentialBackoff`, `WebSocketClient`, `WsState` |
| INFRA-T5 | Heartbeat-Loop: `DomainEvent::HeartbeatSent` alle `HEARTBEAT_INTERVAL_MS` ms über WebSocket senden | 🟠 hoch | ⬜ | `HeartbeatManager`, `WebSocketClient`, `HEARTBEAT_INTERVAL_MS` |

---

### Epic INFRA-E2 · HTTP-Client — Produktionsimplementierung
> Aktuell nur Log-Stub; benötigt für Legacy-REST-Endpunkt `/api/heat_cycles/create`.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| INFRA-T6 | `HttpClient::post_json()` mit `EspHttpConnection` implementieren, TLS-Bundle einbinden | 🟠 hoch | ⬜ | `HttpClient`, `NetworkError`, `BACKEND_HTTP_URL` |
| INFRA-T7 | `HttpClient::get()` implementieren für OTA-Firmware-Versions-Check | 🟡 mittel | ⬜ | `HttpClient`, `OtaService` |
| INFRA-T8 | HTTP-Response-Fehlerbehandlung: 4xx → `NetworkError::HttpError(u16)`, 5xx → Retry-Logik | 🟠 hoch | ⬜ | `HttpClient`, `NetworkError`, `ExponentialBackoff` |

---

### Epic INFRA-E3 · NVS Repositories — Produktionsimplementierung
> Repositories sind strukturell fertig; NvsImpl ist Stub und muss gegen echten Flash-Zugriff ersetzt werden.

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| INFRA-T9 | `HeaterConfigRepository::load()` und `save()` mit `NvsImpl` (echter ESP-IDF-NVS) validieren | 🔴 kritisch | ⬜ | `HeaterConfigRepository`, `NvsDriver`, `NvsImpl` |
| INFRA-T10 | `ConsumptionData` über `HeaterSettingsRepository` in NVS persistieren: `total_cycles`, `total_duration_ms` | 🔴 kritisch | ⬜ | `ConsumptionData`, `HeaterSettingsRepository`, `NvsDriver` |
| INFRA-T11 | NVS-Namespace-Strategie festlegen: ein Namespace pro Repository oder geteilt — entscheiden und dokumentieren | 🟡 mittel | ⬜ | `NvsDriver`, `NvsImpl`, `HeaterConfigRepository`, `HeaterSettingsRepository` |
| INFRA-T12 | Preset-Einstellungen (4 Slots) über Repository laden und speichern | 🟢 niedrig | ⬜ | `HeaterSettingsRepository`, `HeatingMode`, `CORE-T4` |

---

### Epic INFRA-E4 · OTA-Service — Produktionsimplementierung

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| INFRA-T13 | `OtaService::download()` mit `EspOta` implementieren; Chunks schreiben und `OtaProgress`-Events publishen | 🟡 mittel | ⬜ | `OtaService`, `DomainEvent::OtaProgress`, `EventBus` |
| INFRA-T14 | Nach erfolgreichem OTA: `esp_restart()` auslösen und `DomainEvent::OtaCompleted` senden | 🟡 mittel | ⬜ | `OtaService`, `DomainEvent::OtaCompleted` |
| INFRA-T15 | Firmware-Signaturprüfung via ESP32 Secure Boot aktivieren | 🟢 niedrig | ⬜ | `OtaService`, ESP32 Secure Boot V2 |

---

### Epic INFRA-E5 · ClockManager — NTP-Synchronisation

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| INFRA-T16 | `ClockManager::sync_ntp()` mit `EspSntp` implementieren; nach WLAN-Connect aufrufen | 🟠 hoch | ✅ | `ClockManager`, `WifiDriver`, `NTP_SERVER`, `DomainEvent::WifiConnected` |
| INFRA-T17 | Fallback wenn NTP nicht erreichbar: Timestamp aus NVS laden (letzter bekannter Wert) | 🟢 niedrig | ⬜ | `ClockManager`, `NvsDriver`, `NvsError` |

---

## ESP32 — heizbox-esp32

### Epic ESP32-E1 · GPIO-Implementierung

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| ESP32-T1 | `GpioImpl` von HashMap-Stub auf `esp_idf_hal::gpio::PinDriver` umstellen | 🔴 kritisch | ✅ | `GpioImpl`, `GpioDriver`, `pins.rs` |
| ESP32-T2 | Input-Pins (Joystick, Feuer-Taste) mit Pull-Up konfigurieren | 🔴 kritisch | ⬜ | `GpioImpl`, `JOYSTICK_*_PIN`, `FIRE_BTN_PIN` |
| ESP32-T3 | MOSFET-Gate (GPIO 32) als Push-Pull-Output konfigurieren; sicherstellen, dass beim Boot LOW gesetzt wird | 🔴 kritisch | ⬜ | `GpioImpl`, `MOSFET_PIN`, `HeaterController` |
| ESP32-T4 | Interrupt-basiertes Input-Reading für Joystick implementieren (statt Polling) | 🟢 niedrig | ⬜ | `GpioImpl`, `InputHandler`, `GpioDriver` |

---

### Epic ESP32-E2 · I²C-Implementierung (MLX90614 + PCF8574)

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| ESP32-T5 | `I2cImpl` auf `esp_idf_hal::i2c::I2cDriver` umstellen (SCL=27, SDA=26, 100 kHz) | 🔴 kritisch | ✅ | `I2cImpl`, `I2cDriver`, `I2C_SCL_PIN`, `I2C_SDA_PIN` |
| ESP32-T6 | MLX90614-Treiber implementieren: Object-Temp und Ambient-Temp lesen via SMBus-Read-Word | 🔴 kritisch | ⬜ | `I2cImpl`, `SensorError`, `DomainEvent::TemperatureUpdated` |
| ESP32-T7 | Temperatur-Kalibrierung: Emissivitätskorrektur für Metall (Standard 0.95 → anpassen) | 🟡 mittel | ⬜ | `I2cImpl`, MLX90614 Emissivity Register, `SensorError::InvalidCalibration` |
| ESP32-T8 | PCF8574-Treiber implementieren: I²C-Read/Write für GPIO-Expander (8 Bit) | 🟡 mittel | ⬜ | `I2cImpl`, `PCF8574_INT_PIN`, `HAL-T6` |

---

### Epic ESP32-E3 · SPI-Implementierung (ST7789-Display)

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| ESP32-T9 | `SpiImpl` auf `esp_idf_hal::spi::SpiDeviceDriver` umstellen (SPI2/HSPI, 40 MHz) | 🔴 kritisch | ⬜ | `SpiImpl`, `SpiDriver`, `DISPLAY_*_PIN` |
| ESP32-T10 | ST7789-Initialisierungssequenz implementieren: Reset-Puls, Kommandos (COLMOD, MADCTL, CASET, RASET, INVON, DISPON) | 🔴 kritisch | ⬜ | `SpiImpl`, `DisplayManager`, `DISPLAY_DC_PIN`, `DISPLAY_RST_PIN` |
| ESP32-T11 | `FrameBuffer`-Flush via DMA-SPI implementieren für flüssiges Rendering (~20 fps) | 🟠 hoch | ⬜ | `SpiImpl`, `FrameBuffer`, `DisplayManager` |
| ESP32-T12 | Backlight-PWM für Helligkeitssteuerung (GPIO 16) einrichten | 🟢 niedrig | ⬜ | `GpioImpl`, `DISPLAY_BL_PIN`, `DisplayManager` |

---

### Epic ESP32-E4 · NVS-Implementierung

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| ESP32-T13 | `NvsImpl` auf `esp_idf_svc::nvs::EspNvs` umstellen; alle `get_*/set_*`-Methoden mit echtem Flash-Zugriff implementieren | 🔴 kritisch | ⬜ | `NvsImpl`, `NvsDriver`, `NvsError` |
| ESP32-T14 | NVS-Partition in `partitions.csv` prüfen und ggf. vergrößern (Standard 24 KB oft zu klein) | 🟠 hoch | ⬜ | `NvsImpl`, `partitions.csv` |

---

### Epic ESP32-E5 · WLAN-Implementierung

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| ESP32-T15 | `WifiImpl` auf `esp_idf_svc::wifi::EspWifi` umstellen; SSID/Passwort aus NVS laden statt aus `credentials.h` | 🔴 kritisch | ⬜ | `WifiImpl`, `WifiDriver`, `NvsDriver`, `DomainEvent::WifiConnected` |
| ESP32-T16 | Reconnect-Logik bei WLAN-Verbindungsverlust: `DomainEvent::WifiDisconnected` publishen, `ExponentialBackoff` nutzen | 🟠 hoch | ⬜ | `WifiImpl`, `WifiDriver`, `ExponentialBackoff`, `DomainEvent::WifiDisconnected` |
| ESP32-T17 | WLAN-Credentials beim ersten Start via Serial-Konsole oder provisorischem AP-Mode eingeben | 🟢 niedrig | ⬜ | `WifiImpl`, `NvsDriver` |

---

### Epic ESP32-E6 · ADC-Implementierung (Batterie-Monitoring)

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| ESP32-T18 | `AdcImpl` auf `esp_idf_hal::adc::AdcDriver` umstellen; Batteriespannungs-Pin konfigurieren | 🟠 hoch | ⬜ | `AdcImpl`, `AdcDriver`, `CycleResult::voltage_start/end` |
| ESP32-T19 | Spannungsteiler-Kalibrierung: Raw-ADC-Wert in Volt umrechnen (12-Bit ADC, 3.3V Referenz, Teiler-Ratio) | 🟠 hoch | ⬜ | `AdcImpl`, `CycleResult`, `ConsumptionData` |

---

### Epic ESP32-E7 · main.rs — Task-Orchestrierung finalisieren

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| ESP32-T20 | `control_task`: echten Tick-Loop mit `HeaterSm` und MLX90614-Abfrage alle ~220 ms implementieren | 🔴 kritisch | ⬜ | `DeviceApp`, `HeaterSm`, `I2cImpl`, `DomainEvent` |
| ESP32-T21 | `ui_task`: `FrameBuffer`-Render-Loop mit festem Takt (~20 fps, 50 ms) und SPI-Flush implementieren | 🔴 kritisch | ⬜ | `DeviceApp`, `FrameBuffer`, `SpiImpl` |
| ESP32-T22 | `input_task`: GPIO-Polling-Loop (~50 ms) mit `InputHandler` und `EventBus`-Publish implementieren | 🔴 kritisch | ⬜ | `InputHandler`, `GpioImpl`, `EventBus`, `DomainEvent` |
| ESP32-T23 | `network_task`: WLAN-Connect → NTP-Sync → WebSocket-Connect → Reconnect-Loop implementieren | 🔴 kritisch | ⬜ | `WifiImpl`, `ClockManager`, `WebSocketClient`, `DomainEvent` |
| ESP32-T24 | Stack-Größen aller Tasks final anpassen (Netzwerk-Task benötigt deutlich mehr als 8 KB) | 🟠 hoch | ⬜ | `main.rs`, `network_task`, `control_task` |
| ESP32-T25 | Panic-Handler einrichten: bei Panic Fehlermeldung auf Display ausgeben und nach 5 s neu starten | 🟡 mittel | ⬜ | `main.rs`, `DisplayManager`, `esp_idf_svc` |

---

## TESTS — heizbox-tests

### Epic TEST-E1 · Bestehende Tests vervollständigen

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| TEST-T1 | Testfall für `HeaterSm`: Übergang von `Idle` nach `Heating` mit sofortigem Cutoff (0 ms nach Start) | 🟠 hoch | ⬜ | `HeaterSm<Idle>`, `HeaterSm<Heating>`, `HeaterError::CutoffTemperatureExceeded` |
| TEST-T2 | Testfall für `ConsumptionData::record_cycle()`: mehrfaches Aufrufen kumuliert korrekt | 🟠 hoch | ⬜ | `ConsumptionData`, `total_cycles`, `total_duration_ms` |
| TEST-T3 | Testfall für `DomainEvent`-Serialisierung: JSON-Output auf camelCase und korrekte Felder prüfen | 🟠 hoch | ⬜ | `DomainEvent`, `serde_json`, `CORE-T9` |
| TEST-T4 | Testfall für `NavigationFsm`: ungültige Übergänge geben `NavError` zurück | 🟡 mittel | ⬜ | `NavigationFsm`, `NavError`, `ScreenType` |
| TEST-T5 | Testfall für `InputHandler`: kurzer Druck → `Press`, langer Druck → `LongPress` | 🟡 mittel | ⬜ | `InputHandler`, `InputEvent::Press`, `InputEvent::LongPress` |
| TEST-T6 | Testfall für `HeaterConfigRepository` mit Mock-`NvsDriver`: `load()` gibt Default zurück wenn Key fehlt | 🟠 hoch | ⬜ | `HeaterConfigRepository`, `NvsDriver`, `NvsError::KeyNotFound` |
| TEST-T7 | Testfall für `ExponentialBackoff`: Verzögerungen verdoppeln sich korrekt bis zum Maximum | 🟡 mittel | ⬜ | `ExponentialBackoff`, `NetworkError::ReconnectFailed` |

---

### Epic TEST-E2 · Integrationstests Netzwerk & Protokoll

| ID | Aufgabe | Priorität | Status | Typen / Referenzen |
|---|---|---|---|---|
| TEST-T8 | Mock-WebSocket-Server für Tests aufsetzen; `WebSocketClient` gegen echten WS-Handshake testen (Host-seitig) | 🟢 niedrig | ⬜ | `WebSocketClient`, `DomainEvent`, `WsState` |
| TEST-T9 | Roundtrip-Test: `DomainEvent` serialisieren → als String → deserialisieren → Gleichheit prüfen | 🟠 hoch | ⬜ | `DomainEvent`, `serde_json`, `CORE-T10` |

---

## Zusammenfassung

| Crate | Kritisch | Hoch | Mittel | Niedrig | Gesamt |
|---|---|---|---|---|---|
| heizbox-core | 3 | 5 | 4 | 1 | **13** |
| heizbox-hal | 2 | 3 | 2 | 1 | **8** |
| heizbox-app | 5 | 4 | 4 | 1 | **14** |
| heizbox-infra | 5 | 8 | 4 | 3 | **20** |
| heizbox-esp32 | 12 | 7 | 4 | 3 | **26** |
| heizbox-tests | 0 | 5 | 4 | 2 | **11** |
| **Gesamt** | **27** | **32** | **22** | **11** | **92** |

> **Kritischer Pfad für erste Hardware-Inbetriebnahme:**  
> ESP32-T1 → ESP32-T5 → ESP32-T6 → ESP32-T9 → ESP32-T10 → ESP32-T13 → ESP32-T15 → APP-T11 → APP-T12 → APP-T13 → ESP32-T20 → ESP32-T21 → ESP32-T22 → ESP32-T23
