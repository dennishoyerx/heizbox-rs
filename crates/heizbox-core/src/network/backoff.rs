//! TEST-T7 / CORE: Pure-logic exponential back-off — no platform dependencies,
//! so it can be unit-tested on the host without pulling in the esp-idf chain.

/// Exponential back-off for connection retries.
///
/// Starts at `initial_ms`, doubles on each [`next_delay_ms`] call, capped at
/// `max_ms`.  Call [`reset`] after a successful connection.
pub struct ExponentialBackoff {
    current_ms: u32,
    initial_ms: u32,
    max_ms:     u32,
}

impl ExponentialBackoff {
    /// Default: 100 ms initial, 30 s maximum.
    pub fn default_config() -> Self {
        Self::new_with(100, 30_000)
    }

    /// Create with custom initial / maximum delays.
    pub fn new(initial_ms: u32, max_ms: u32) -> Self {
        Self::new_with(initial_ms, max_ms)
    }

    /// Custom initial / maximum delays.
    pub fn new_with(initial_ms: u32, max_ms: u32) -> Self {
        Self { current_ms: initial_ms, initial_ms, max_ms }
    }

    /// Returns the current delay (ms) then doubles for the next call.
    pub fn next_delay_ms(&mut self) -> u32 {
        let delay = self.current_ms;
        self.current_ms = self.current_ms.saturating_mul(2).min(self.max_ms);
        delay
    }

    /// Reset to the initial delay (call after a successful connection).
    pub fn reset(&mut self) {
        self.current_ms = self.initial_ms;
    }

    /// Current wait interval without advancing.
    pub fn current_ms(&self) -> u32 {
        self.current_ms
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(100, 30_000)
    }
}
