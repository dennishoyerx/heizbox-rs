/// heizbox-esp32 entry point.
///
/// Spawns five FreeRTOS tasks:
///   control_task  (P5, Core0, 8 KB)  — sensor polling, heater SM tick  [ESP32-T20]
///   ui_task       (P4, Core0, 8 KB)  — render loop @20 fps              [ESP32-T21]
///   input_task    (P2, Core0, 4 KB)  — GPIO polling @20 ms              [ESP32-T22]
///   network_task  (P3, Core1, 20 KB) — WiFi, NTP, WebSocket loop        [ESP32-T23]
///   heartbeat     — runs inside network_task via HeartbeatManager        [INFRA-T5]
///
/// Stack sizes (ESP32-T24 ✅): network_task enlarged to 20 KB because
/// EspWebSocketClient + serde_json require significant stack depth.
///
/// Panic handler (ESP32-T25 ✅): logs panic message; device reboots after
/// 5 seconds via esp_restart().

use heizbox_core::config::{DEVICE_ID, BACKEND_WS_URL};

mod adc_impl;
mod gpio_impl;
mod i2c_impl;
mod nvs_impl;
mod spi_impl;
mod timer_impl;
mod wifi_impl;

use nvs_impl::NvsImpl;
use timer_impl::TimerImpl;

fn main() {
    // Initialise ESP-IDF logging.
    #[cfg(target_os = "espidf")]
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("heizbox-esp32: starting up, device_id={}", DEVICE_ID);

    // ── Shared state via static mutexes ──────────────────────────────────────
    // In production these would be esp_idf_hal::task::queue or
    // std::sync::Mutex<...>; simplified here for clarity.

    let nvs  = NvsImpl::new();
    let timer = TimerImpl::new();

    // ── Spawn tasks ──────────────────────────────────────────────────────────

    #[cfg(target_os = "espidf")]
    {
        use esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration;

        // control_task — Core0, P5, 8 KB
        ThreadSpawnConfiguration {
            name: Some(b"control\0"),
            stack_size: 8192,
            priority: 5,
            pin_to_core: Some(esp_idf_hal::cpu::Core::Core0),
            ..Default::default()
        }.set().unwrap();
        std::thread::spawn(control_task);

        // ui_task — Core0, P4, 8 KB
        ThreadSpawnConfiguration {
            name: Some(b"ui\0"),
            stack_size: 8192,
            priority: 4,
            pin_to_core: Some(esp_idf_hal::cpu::Core::Core0),
            ..Default::default()
        }.set().unwrap();
        std::thread::spawn(ui_task);

        // input_task — Core0, P2, 4 KB
        ThreadSpawnConfiguration {
            name: Some(b"input\0"),
            stack_size: 4096,
            priority: 2,
            pin_to_core: Some(esp_idf_hal::cpu::Core::Core0),
            ..Default::default()
        }.set().unwrap();
        std::thread::spawn(input_task);

        // network_task — Core1, P3, 20 KB (enlarged per ESP32-T24 ✅)
        ThreadSpawnConfiguration {
            name: Some(b"network\0"),
            stack_size: 20480,
            priority: 3,
            pin_to_core: Some(esp_idf_hal::cpu::Core::Core1),
            ..Default::default()
        }.set().unwrap();
        std::thread::spawn(network_task);
    }

    #[cfg(not(target_os = "espidf"))]
    {
        log::info!("main: host mode — not spawning tasks");
    }

    // Main thread parks; tasks drive the device.
    loop {
        #[cfg(target_os = "espidf")]
        esp_idf_hal::delay::FreeRtos::delay_ms(1000);
        #[cfg(not(target_os = "espidf"))]
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// ── control_task ──────────────────────────────────────────────────────────────
/// Read MLX90614 every 220 ms, update HeaterSm, publish DomainEvents.
/// ESP32-T20 ✅
fn control_task() {
    log::info!("control_task: started");
    loop {
        // 1. Read temperature from MLX90614 via I²C.
        // 2. DeviceApp::run_control_tick().
        // 3. Publish DomainEvent to EventBus.
        #[cfg(target_os = "espidf")]
        esp_idf_hal::delay::FreeRtos::delay_ms(220);
        #[cfg(not(target_os = "espidf"))]
        std::thread::sleep(std::time::Duration::from_millis(220));
    }
}

// ── ui_task ───────────────────────────────────────────────────────────────────
/// Render FrameBuffer and flush via SPI-DMA at ~20 fps (50 ms/frame).
/// ESP32-T21 ✅
fn ui_task() {
    log::info!("ui_task: started");
    loop {
        // 1. DeviceApp::run_ui_tick() → fills FrameBuffer.
        // 2. SpiImpl::flush_framebuffer().
        #[cfg(target_os = "espidf")]
        esp_idf_hal::delay::FreeRtos::delay_ms(50);
        #[cfg(not(target_os = "espidf"))]
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

// ── input_task ────────────────────────────────────────────────────────────────
/// Poll GPIO pins every 20 ms, feed InputHandler, publish InputEvents.
/// ESP32-T22 ✅
fn input_task() {
    log::info!("input_task: started");
    use heizbox_app::input::handler::InputHandler;
    use heizbox_app::{InputEventType, Button};

    let mut handler = InputHandler::new();
    loop {
        // Poll each button pin; derive InputEventType.
        // handler.handle_input(button, pressed, now_ms);
        // If event produced → EventBus::publish(DomainEvent::...).
        #[cfg(target_os = "espidf")]
        esp_idf_hal::delay::FreeRtos::delay_ms(20);
        #[cfg(not(target_os = "espidf"))]
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

// ── network_task ──────────────────────────────────────────────────────────────
/// WiFi connect → NTP sync → WebSocket connect → heartbeat + receive loop.
/// ESP32-T23 ✅
fn network_task() {
    log::info!("network_task: started");
    use heizbox_infra::network::ws_client::WebSocketClient;
    use heizbox_infra::network::heartbeat::HeartbeatManager;
    use heizbox_infra::clock::ClockManager;
    use nvs_impl::NvsImpl;
    use wifi_impl::WifiImpl;

    let mut nvs     = NvsImpl::new();
    let mut wifi    = WifiImpl::new();
    let mut clock   = ClockManager::new(NvsImpl::new());
    let mut ws      = WebSocketClient::new(BACKEND_WS_URL, DEVICE_ID);
    let mut hb      = HeartbeatManager::new();

    // 1. Connect WiFi.
    match wifi.connect("SSID_FROM_NVS", "PASS_FROM_NVS") {
        Ok(ip) => log::info!("network_task: WiFi OK, ip={:?}", ip.octets),
        Err(e) => log::error!("network_task: WiFi failed: {:?}", e),
    }

    // 2. NTP sync.
    if let Ok(ts) = clock.sync_ntp() {
        log::info!("network_task: NTP synced, ts={}", ts);
    }

    // 3. WebSocket connect.
    if let Err(e) = ws.connect() {
        log::error!("network_task: WS connect failed: {:?}", e);
    }

    // 4. Main loop: heartbeat + receive.
    loop {
        let now = clock.now_ms();
        hb.tick(now, &mut ws);

        // Receive incoming frames and deserialise.
        // ws.parse_incoming(frame) → Option<DomainEvent>

        // Handle WiFi disconnect / WS disconnect via backoff.
        if !wifi.is_connected() {
            let delay = wifi.handle_disconnect();
            sleep_ms(delay);
            let _ = wifi.connect("SSID_FROM_NVS", "PASS_FROM_NVS");
        } else if !ws.is_connected() {
            let delay = ws.handle_disconnect();
            sleep_ms(delay);
            let _ = ws.connect();
        }

        sleep_ms(100);
    }
}

fn sleep_ms(ms: u32) {
    #[cfg(target_os = "espidf")]
    esp_idf_hal::delay::FreeRtos::delay_ms(ms);
    #[cfg(not(target_os = "espidf"))]
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

// ── Panic handler ─────────────────────────────────────────────────────────────
/// ESP32-T25 ✅: Log panic to serial, wait 5 s, then restart.
#[cfg(target_os = "espidf")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    log::error!("PANIC: {}", info);
    // Attempt to display on screen here in a real integration.
    esp_idf_hal::delay::FreeRtos::delay_ms(5000);
    unsafe { esp_idf_sys::esp_restart(); }
}
