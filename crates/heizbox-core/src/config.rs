/// Temperature above which heating is forcibly stopped.
/// Value `420` disables the cutoff check entirely (DynaVap full-extraction mode).
pub const CUTOFF_DISABLED: u16 = 420;

// ── Default operating parameters (verified against DynaVap M 2021 specs) ────

/// Default target temperature in °C.
pub const DEFAULT_TARGET_TEMP: u16 = 200;

/// Default auto-stop timeout in milliseconds (90 s is safe for all DynaVap tips).
pub const DEFAULT_AUTO_STOP_MS: u32 = 90_000;

/// Default heater power level (percentage 0–100).
pub const DEFAULT_POWER: u8 = 100;

// ── Temperature presets ───────────────────────────────────────────────────────
/// Flavour mode: lower temp, more terpenes, lighter vapour.
pub const PRESET_FLAVOR: u16 = 185;
/// Balanced mode: all-rounder.
pub const PRESET_BALANCED: u16 = 200;
/// Extraction mode: more vapour, slightly less flavour.
pub const PRESET_EXTRACTION: u16 = 210;
/// Full-extraction mode: maximum vapour density.
pub const PRESET_FULL: u16 = 220;

/// Screensaver idle timeout in milliseconds (60 s).
pub const SCREENSAVER_TIMEOUT_MS: u32 = 60_000;

/// Heartbeat interval in milliseconds (30 s).
pub const HEARTBEAT_INTERVAL_MS: u32 = 30_000;

/// NTP server hostname.
pub const NTP_SERVER: &str = "pool.ntp.org";

/// Device ID — must match the backend `deviceId` query parameter.
pub const DEVICE_ID: &str = "heizbox-001";

/// Backend WebSocket URL (scheme + host only; query params added at connect time).
pub const BACKEND_WS_URL: &str = "wss://heizbox.workers.dev/ws/status";

/// Backend HTTP base URL for REST endpoints.
pub const BACKEND_HTTP_URL: &str = "https://heizbox.workers.dev";
