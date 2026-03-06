//! Pin definitions for the ESP32 induction heater.
//! Derived from C++ Config.h. Several conflicts are noted below — hardware
//! review required before using conflicting pins simultaneously.

#![allow(dead_code)]

pub mod display {
    // Using SPI2 (HSPI) pins: SCK=14, MOSI=12. MISO not used.
    pub const MOSI: u8 = 12;
    pub const SCK: u8 = 14;
    pub const CS: u8 = 5;   // Chip-select handled via GPIO (not hardware)
    pub const DC: u8 = 4;   // Data/Command
    pub const RST: u8 = 15; // Reset
    /// Backlight — no conflict.
    pub const BL: u8 = 16;
    pub const WIDTH: u16 = 240;
    pub const HEIGHT: u16 = 280;
}

/// 5-way joystick GPIO pins.
pub mod joystick {
    pub const UP: u8 = 1;
    pub const DOWN: u8 = 0;
    pub const LEFT: u8 = 3;
    /// ⚠ Conflicts with STATUS_LED (pin 2).
    pub const RIGHT: u8 = 2;
    pub const PRESS: u8 = 4;
}

/// Physical fire button.
pub const FIRE_BUTTON: u8 = 13;

pub mod rotary {
    pub const CLK: u8 = 21;
    pub const SW: u8 = 22;
    pub const DT: u8 = 19;
}

pub mod pcf8574 {
    pub const SCL: u8 = 27;
    pub const SDA: u8 = 26;
    /// ⚠ Conflicts with SPEAKER (pin 25).
    pub const INT_PIN: u8 = 25;
}

pub mod heater {
    pub const MOSFET_GATE: u8 = 32;
}

/// Thermocouple MAX6675 shares SCK with the display bus.
pub mod thermocouple {
    pub const SCK: u8 = 18; // shared with display
    // CS and SO TBD — avoid conflict with MOSFET_GATE (32).
}

/// ⚠ Conflicts with joystick RIGHT (pin 2).
pub const STATUS_LED: u8 = 2;
/// ⚠ Conflicts with PCF8574 INT_PIN (pin 25).
pub const SPEAKER: u8 = 25;

pub const I2C_BUS: u8 = 0;
pub const SPI_BUS: u8 = 2;

pub use display::BL as TFT_BL;
