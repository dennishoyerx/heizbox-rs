/// Temperature above which heating is forcibly stopped (special value = disabled).
pub const CUTOFF_DISABLED: u16 = 420;

/// Default target temperature in °C.
pub const DEFAULT_TARGET_TEMP: u16 = 200;

/// Default auto-stop timeout in milliseconds (90 s).
pub const DEFAULT_AUTO_STOP_MS: u32 = 90_000;

/// Default heater power (percentage 0–100).
pub const DEFAULT_POWER: u8 = 100;

/// Temperature preset slots.
pub const PRESET_FLAVOR: u16 = 185;
pub const PRESET_BALANCED: u16 = 200;
pub const PRESET_EXTRACTION: u16 = 210;
pub const PRESET_FULL: u16 = 220;

/// Screensaver idle timeout in milliseconds.
pub const SCREENSAVER_TIMEOUT_MS: u32 = 60_000;
