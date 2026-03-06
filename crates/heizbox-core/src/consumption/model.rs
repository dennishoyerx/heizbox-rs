use serde::{Deserialize, Serialize};

/// Aggregated consumption data stored across sessions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsumptionData {
    /// Total number of completed heat cycles.
    pub total_cycles: u32,
    /// Cumulative heating duration in milliseconds.
    pub total_duration_ms: u64,
}

impl ConsumptionData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a completed cycle.
    pub fn record_cycle(&mut self, duration_ms: u32) {
        self.total_cycles += 1;
        self.total_duration_ms += duration_ms as u64;
    }

    /// Total heating time in seconds.
    pub fn total_duration_secs(&self) -> u64 {
        self.total_duration_ms / 1000
    }
}
