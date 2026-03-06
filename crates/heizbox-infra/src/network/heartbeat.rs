/// Tracks when the last heartbeat was sent and whether one is due.
pub struct HeartbeatManager {
    interval_ms: u32,
    last_sent_ms: u32,
}

impl HeartbeatManager {
    pub fn new(interval_ms: u32) -> Self {
        Self {
            interval_ms,
            last_sent_ms: 0,
        }
    }

    /// Returns `true` if a heartbeat should be sent now.
    pub fn is_due(&self, now_ms: u32) -> bool {
        now_ms.saturating_sub(self.last_sent_ms) >= self.interval_ms
    }

    /// Record that a heartbeat was sent at `now_ms`.
    pub fn mark_sent(&mut self, now_ms: u32) {
        self.last_sent_ms = now_ms;
    }
}
