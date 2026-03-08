/// Heartbeat manager: publishes DomainEvent::HeartbeatSent via WebSocket
/// every HEARTBEAT_INTERVAL_MS milliseconds.
///
/// INFRA-T5 ✅

use heizbox_core::event::DomainEvent;
use heizbox_core::config::HEARTBEAT_INTERVAL_MS;
use super::ws_client::WebSocketClient;

pub struct HeartbeatManager {
    last_sent_ms: u64,
}

impl HeartbeatManager {
    pub fn new() -> Self {
        Self { last_sent_ms: 0 }
    }

    /// Call from the network task loop.  `now_ms` is the current wall-clock
    /// time in milliseconds.  Sends a heartbeat if the interval has elapsed.
    pub fn tick(&mut self, now_ms: u64, ws: &mut WebSocketClient) {
        if now_ms.saturating_sub(self.last_sent_ms) >= HEARTBEAT_INTERVAL_MS as u64 {
            if ws.is_connected() {
                let event = DomainEvent::HeartbeatSent;
                match ws.send_event(&event) {
                    Ok(()) => {
                        self.last_sent_ms = now_ms;
                        log::debug!("HeartbeatManager: heartbeat sent");
                    }
                    Err(e) => {
                        log::warn!("HeartbeatManager: send failed: {:?}", e);
                    }
                }
            }
        }
    }
}

impl Default for HeartbeatManager {
    fn default() -> Self { Self::new() }
}
