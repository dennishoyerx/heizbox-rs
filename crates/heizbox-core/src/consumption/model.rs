use serde::{Deserialize, Serialize};

/// Aggregated lifetime consumption statistics stored in NVS.
///
/// CORE-T6: `record_cycle` is designed to be called exactly once per completed
/// cycle.  The caller (DeviceApp) is responsible for ensuring it is not invoked
/// more than once per finalized `CycleResult` (no retry path should call it).
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

    /// Record one completed cycle.  Must be called exactly once per cycle.
    pub fn record_cycle(&mut self, duration_ms: u32) {
        self.total_cycles = self.total_cycles.saturating_add(1);
        self.total_duration_ms = self.total_duration_ms.saturating_add(duration_ms as u64);
    }

    /// Total heating time in whole seconds.
    pub fn total_duration_secs(&self) -> u64 {
        self.total_duration_ms / 1_000
    }
}
