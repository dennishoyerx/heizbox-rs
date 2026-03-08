use serde::{Deserialize, Serialize};
use crate::heater::CycleResult;

<<<<<<< ours
=======
/// All domain events serialise with a `type` discriminant in `camelCase`
/// so they map directly to the WebSocket protocol (CORE-T9).
>>>>>>> theirs
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
<<<<<<< ours
=======
        /// ESP-IDF disconnect reason code.
>>>>>>> theirs
        reason: u8,
    },
    WebSocketConnected,
    WebSocketDisconnected,

<<<<<<< ours
=======
    // ── Session (CORE-T7) ────────────────────────────────────────────────────
    /// Mirrors the WebSocket `sessionUpdate` message sent by the device.
    /// `clicks`        – number of heat-button presses this session.
    /// `last_click`    – UNIX timestamp of the most recent click.
    /// `session_start` – UNIX timestamp when the current session began.
    SessionUpdate {
        clicks: u32,
        last_click: u32,
        session_start: u32,
    },

>>>>>>> theirs
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
<<<<<<< ours
}

=======

    // ── Internal ────────────────────────────────────────────────────────────
    /// Emitted each time a heartbeat packet is dispatched (CORE-T8).
    HeartbeatSent,

    // ── WebSocket protocol messages ──────────────────────────────────────────
    /// Mirrors the WebSocket `heatCycleCompleted` message from the device.
    HeatCycleCompleted {
        duration_ms: u32,
        cycle: u32,
    },
    /// Mirrors the WebSocket `statusUpdate` message (isOn, isHeating).
    StatusUpdate {
        is_on: bool,
        is_heating: bool,
    },
}

/// Error sub-events emitted when the heater safety system triggers.
/// Fields use `camelCase` in JSON (CORE-T9).
>>>>>>> theirs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HeaterErrorEvent {
    CutoffExceeded { temp: u16, limit: u16 },
    TimeoutExceeded { duration: u32, limit: u32 },
    InvalidReading { reason: &'static str },
}
