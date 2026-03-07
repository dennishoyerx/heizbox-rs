use esp_idf_svc::sys::link_patches;
use log::{info, warn};
use std::{sync::{Arc, Mutex}, thread, time::Duration};

mod config;
mod error;
mod hal_impl;
mod display_manager;

use heizbox_app::device::DeviceApp;
use heizbox_core::event::DomainEvent;
use heizbox_hal::{GpioDriver, I2cDriver, NvsDriver, SpiDriver, WifiDriver, sensors::mlx90614::Mlx90614};
use heizbox_infra::clock::ClockManager;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use crate::display_manager::DisplayManager;

fn main() -> anyhow::Result<()> {
    // Required by esp-idf-svc.
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Heizbox starting — device_id={}", config::DEVICE_ID);

    // ── Take ESP32 peripherals (once) ────────────────────────────────────────
    let peripherals = Peripherals::take().map_err(|_| anyhow::anyhow!("Failed to take peripherals"))?;
    let i2c0 = peripherals.i2c0;
    // Extract the GPIO pins needed for I2C (moving them out of peripherals)
    let pins = peripherals.pins;
    let sda = pins.gpio26;
    let scl = pins.gpio27;

    // SPI2 pins for display (HSPI)
    let spi2 = peripherals.spi2;
    let spi_sck = pins.gpio14;
    let spi_mosi = pins.gpio12;

    // WiFi peripherals
    let modem = peripherals.modem;
    let sysloop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;

    // ── Initialise HAL drivers ─────────────────────────────────────────────
    let gpio = hal_impl::GpioImpl::new()?;
    let i2c_impl = hal_impl::I2cImpl::new(i2c0, sda, scl)?;
    let i2c: Box<dyn I2cDriver + Send> = Box::new(i2c_impl);
    let spi_impl = hal_impl::SpiImpl::new(spi2, spi_sck, spi_mosi)?;
    let spi: Box<dyn SpiDriver + Send> = Box::new(spi_impl);
    let wifi = hal_impl::WifiImpl::new(modem, sysloop)?;
    let adc = hal_impl::AdcImpl::new();

    // ── Initialise display ────────────────────────────────────────────────
    let gpio_driver: Box<dyn GpioDriver + Send> = Box::new(gpio);
    let mut display_manager = DisplayManager::new(spi, gpio_driver, 240, 280);
    if let Err(e) = display_manager.init() {
        eprintln!("Display init failed: {:?}", e);
    } else {
        info!("Display initialized");
    }

    // ── Initialise sensors ───────────────────────────────────────────────
    let mlx90614 = Mlx90614::new(i2c, 0x5A);

    // ── Application ───────────────────────────────────────────────────────
    let app = Arc::new(Mutex::new(DeviceApp::with_sensor(mlx90614)));

    info!("All drivers initialised — spawning tasks");

    // ── Spawn FreeRTOS tasks ──────────────────────────────────────────────
    let app_control = Arc::clone(&app);
    thread::Builder::new()
        .name("control".into())
        .stack_size(8 * 1024)
        .spawn(move || control_task(app_control))
        .unwrap();

    let app_network = Arc::clone(&app);
    thread::Builder::new()
        .name("network".into())
        .stack_size(8 * 1024)
        .spawn(move || network_task(app_network, wifi))
        .unwrap();

    let app_ui = Arc::clone(&app);
    thread::Builder::new()
        .name("ui".into())
        .stack_size(8 * 1024)
        .spawn(move || ui_task(app_ui))
        .unwrap();

    let app_input = Arc::clone(&app);
    thread::Builder::new()
        .name("input".into())
        .stack_size(4 * 1024)
        .spawn(move || input_task(app_input))
        .unwrap();

    // Main thread just keeps the watchdog happy.
    loop {
        thread::sleep(Duration::from_secs(5));
        info!("Heartbeat — main thread alive");
    }
}

// ── Task bodies ───────────────────────────────────────────────────────────────

fn control_task(app: Arc<Mutex<DeviceApp>>) {
    info!("[control] task started");
    loop {
        let mut app_guard = app.lock().unwrap();
        app_guard.update_heater();
        app_guard.update_sensors();
        // In a full implementation, we would also pop events and dispatch them.
        drop(app_guard);
        thread::sleep(Duration::from_millis(100));
    }
}

fn network_task(app: Arc<Mutex<DeviceApp>>, mut wifi: impl WifiDriver) {
    info!("[network] task started");

    let mut clock = ClockManager::new();

    loop {
        if !wifi.is_connected() {
            info!("Connecting to WiFi...");
            match wifi.connect(config::WIFI_SSID, config::WIFI_PASSWORD) {
                Ok(()) => {
                    info!("WiFi connected");
                    // Publish WifiConnected event
                    if let Some(ssid) = wifi.ssid() {
                        let _ = app.lock().unwrap().push_event(DomainEvent::WifiConnected {
                            ssid: ssid.to_string(),
                        });
                    } else {
                        // Fallback, should not happen
                        let _ = app.lock().unwrap().push_event(DomainEvent::WifiConnected {
                            ssid: "unknown".to_string(),
                        });
                    }
                    // After successful connection, synchronize NTP clock
                    match clock.sync_ntp() {
                        Ok(()) => {
                            info!("NTP synchronized, current UTC time: {}", clock.now_unix());
                        }
                        Err(e) => {
                            warn!("NTP synchronization failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("WiFi connection failed: {:?}", e);
                }
            }
        } else {
            // Already connected: if clock not synced, try again periodically
            if !clock.is_synced() {
                if let Err(e) = clock.sync_ntp() {
                    warn!("NTP retry failed: {}", e);
                } else {
                    info!("NTP synchronized on retry, current UTC time: {}", clock.now_unix());
                }
            }
        }

        // Additional network maintenance (WebSocket, heartbeats, OTA checks) would go here.

        thread::sleep(Duration::from_secs(1));
    }
}

fn ui_task(app: Arc<Mutex<DeviceApp>>) {
    info!("[ui] task started");
    loop {
        let mut app_guard = app.lock().unwrap();
        app_guard.render();
        drop(app_guard);
        thread::sleep(Duration::from_millis(50));
    }
}

fn input_task(app: Arc<Mutex<DeviceApp>>) {
    info!("[input] task started");
    loop {
        let mut app_guard = app.lock().unwrap();
        // Placeholder: read input events and call app_guard.handle_input(event)
        drop(app_guard);
        thread::sleep(Duration::from_millis(20));
    }
}
