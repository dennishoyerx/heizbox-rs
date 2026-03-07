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
use heizbox_infra::network::reconnect::ExponentialBackoff;
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
    let nvs = Arc::new(
        hal_impl::NvsImpl::new()
            .map_err(|e| anyhow::anyhow!("NVS initialization failed: {:?}", e))?
    );
    let wifi = hal_impl::WifiImpl::new(modem, sysloop, Arc::clone(&nvs))?;
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
    let nvs_network = Arc::clone(&nvs);
    thread::Builder::new()
        .name("network".into())
        .stack_size(8 * 1024)
        .spawn(move || network_task(app_network, wifi, nvs_network))
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

fn network_task(
    app: Arc<Mutex<DeviceApp>>,
    mut wifi: impl WifiDriver,
    nvs: Arc<hal_impl::NvsImpl>,
) {
    info!("[network] task started");

    let mut clock = ClockManager::new();
    let mut backoff = ExponentialBackoff::new_with(2000, 60000);
    let mut failure_count = 0;
    const MAX_RETRIES: u32 = 5;

    loop {
        if !wifi.is_connected() {
            // Load credentials from NVS
            let ssid = match nvs.get_str("wifi", "ssid") {
                Ok(s) if !s.is_empty() => s,
                Ok(_) => {
                    warn!("WiFi SSID not configured in NVS. Waiting for credentials...");
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
                Err(_) => {
                    warn!("WiFi SSID not found in NVS. Waiting for credentials...");
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };
            let password = match nvs.get_str("wifi", "password") {
                Ok(s) => s,
                Ok(_) => {
                    warn!("WiFi password not configured in NVS. Waiting for credentials...");
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
                Err(_) => {
                    warn!("WiFi password not found in NVS. Waiting for credentials...");
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };

            info!("Attempting WiFi connection to SSID: {}", ssid);
            match wifi.connect(&ssid, &password) {
                Ok(()) => {
                    info!("WiFi connected");
                    failure_count = 0;
                    backoff.reset();

                    // Publish WifiConnected event
                    if let Some(connected_ssid) = wifi.ssid() {
                        let _ = app.lock().unwrap().push_event(DomainEvent::WifiConnected {
                            ssid: connected_ssid.to_string(),
                        });
                    } else {
                        let _ = app.lock().unwrap().push_event(DomainEvent::WifiConnected {
                            ssid: ssid.clone(),
                        });
                    }

                    // Synchronize NTP clock
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
                    failure_count += 1;
                    if failure_count >= MAX_RETRIES {
                        warn!("Maximum reconnect retries reached, publishing WifiDisconnected");
                        let _ = app.lock().unwrap().push_event(DomainEvent::WifiDisconnected {
                            reason: 1, // Connection failure
                        });
                        backoff.reset();
                        failure_count = 0;
                    }
                    let delay_ms = backoff.next_delay_ms();
                    thread::sleep(Duration::from_millis(delay_ms as u64));
                    continue; // Skip the default sleep at end of loop
                }
            }
        } else {
            // Already connected: ensure NTP is synced periodically
            if !clock.is_synced() {
                if let Err(e) = clock.sync_ntp() {
                    warn!("NTP retry failed: {}", e);
                } else {
                    info!("NTP synchronized on retry, current UTC time: {}", clock.now_unix());
                }
            }
        }

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
