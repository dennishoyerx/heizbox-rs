/// Unique identifier for this device (used as WebSocket client-id).
pub const DEVICE_ID: &str = "heizbox-01";

/// WebSocket backend URL.
pub const BACKEND_WS_URL: &str = "wss://backend.hzbx.de/ws/status";

/// REST backend base URL.
pub const BACKEND_HTTP_URL: &str = "https://backend.hzbx.de";

/// NTP server.
pub const NTP_SERVER: &str = "pool.ntp.org";

/// Heartbeat interval in milliseconds.
pub const HEARTBEAT_INTERVAL_MS: u32 = 5_000;
