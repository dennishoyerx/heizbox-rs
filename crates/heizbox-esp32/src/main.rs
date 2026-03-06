use esp_idf_svc::sys::link_patches;
use log::{info, warn};
use std::{thread, time::Duration};

mod config;
mod error;
mod hal_impl;

use heizbox_app::device::DeviceApp;
use heizbox_hal::{GpioDriver, I2cDriver, NvsDriver, SpiDriver, WifiDriver};
use heizbox_infra::clock::ClockManager;
use esp_idf_hal::peripherals::Peripherals;

fn main() -> anyhow::Result<()> {
    // Required by esp-idf-svc.
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Heizbox starting — device_id={}", config::DEVICE_ID);

    // ── Take ESP32 peripherals (once) ────────────────────────────────────────
    let peripherals = Peripherals::take().map_err(|_| anyhow::anyhow!("Failed to take peripherals"))?;
    let i2c0 = peripherals.i2c0;
    // Extract the GPIO pins needed for I2C (moving them out of peripherals)
    let sda = peripherals.pins.gpio26;
    let scl = peripherals.pins.gpio27;

    // ── Initialise HAL drivers ─────────────────────────────────────────────
    let gpio = hal_impl::GpioImpl::new()?;
    // I2C: SDA=GPIO26, SCL=GPIO27
    let i2c = hal_impl::I2cImpl::new(i2c0, sda, scl)?;
    // SPI: currently stub
    let spi = hal_impl::SpiImpl::new();
    let wifi = hal_impl::WifiImpl::new();
    let adc = hal_impl::AdcImpl::new();
    // NVS: may need adjustment later; stub for now
    let nvs = hal_impl::NvsImpl::new()?;

    // ── Application ───────────────────────────────────────────────────────
    let app = DeviceApp::new();

    info!("All drivers initialised — spawning tasks");

    // ── Spawn FreeRTOS tasks ──────────────────────────────────────────────
    thread::Builder::new()
        .name("control".into())
        .stack_size(8 * 1024)
        .spawn(control_task)
        .unwrap();

    thread::Builder::new()
        .name("network".into())
        .stack_size(8 * 1024)
        .spawn(network_task)
        .unwrap();

    thread::Builder::new()
        .name("ui".into())
        .stack_size(8 * 1024)
        .spawn(ui_task)
        .unwrap();

    thread::Builder::new()
        .name("input".into())
        .stack_size(4 * 1024)
        .spawn(input_task)
        .unwrap();

    // Main thread just keeps the watchdog happy.
    loop {
        thread::sleep(Duration::from_secs(5));
        info!("Heartbeat — main thread alive");
    }
}

// ── Task bodies ───────────────────────────────────────────────────────────────

fn control_task() {
    info!("[control] task started");
    loop {
        // TODO: tick HeaterSm, read sensors, push DomainEvents
        thread::sleep(Duration::from_millis(100));
    }
}

fn network_task() {
    info!("[network] task started");

    // Initialize WiFi driver and ClockManager
    let mut wifi = hal_impl::WifiImpl::new();
    let mut clock = ClockManager::new();

    // WiFi credentials (should be sourced from config or NVS in production)
    let ssid = "your-ssid";
    let password = "your-password";

    loop {
        if !wifi.is_connected() {
            // Attempt to connect
            info!("Connecting to WiFi...");
            match wifi.connect(ssid, password) {
                Ok(()) => {
                    info!("WiFi connected");
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

fn ui_task() {
    info!("[ui] task started");
    loop {
        // TODO: render active Screen to TFT
        thread::sleep(Duration::from_millis(50));
    }
}

fn input_task() {
    info!("[input] task started");
    loop {
        // TODO: poll GPIO joystick, dispatch InputEvents
        thread::sleep(Duration::from_millis(20));
    }
}
