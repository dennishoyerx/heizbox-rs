use serde::{Deserialize, Serialize};
use crate::heater::CycleResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DomainEvent {
    // ── Heater ──────────────────────────────────────────────────────────────
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

    // ── Temperature ─────────────────────────────────────────────────────────
    TemperatureUpdated {
        current: u16,
        ambient: u16,
        raw_ir: u16,
    },

    // ── Network ─────────────────────────────────────────────────────────────
    WifiConnected {
        ssid: String,
    },
    WifiDisconnected {
        reason: u8,
    },
    WebSocketConnected,
    WebSocketDisconnected,

    // ── Persistence ─────────────────────────────────────────────────────────
    SettingsPersisted {
        key: &'static str,
    },

    // ── OTA ─────────────────────────────────────────────────────────────────
    OtaStarted,
    OtaProgress {
        percent: u8,
    },
    OtaCompleted,
    OtaFailed {
        reason: &'static str,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HeaterErrorEvent {
    CutoffExceeded { temp: u16, limit: u16 },
    TimeoutExceeded { duration: u32, limit: u32 },
    InvalidReading { reason: &'static str },
}
