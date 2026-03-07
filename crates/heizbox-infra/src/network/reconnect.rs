/// Exponential back-off for connection retries.
pub struct ExponentialBackoff {
    current_ms: u32,
    initial_ms: u32,
    max_ms: u32,
}

impl ExponentialBackoff {
    pub fn new() -> Self {
        Self {
            current_ms: 100,
            initial_ms: 100,
            max_ms: 30_000,
        }
    }

    /// Create a new ExponentialBackoff with custom initial and maximum delays.
    pub fn new_with(initial_ms: u32, max_ms: u32) -> Self {
        Self {
            current_ms: initial_ms,
            initial_ms,
            max_ms,
        }
    }

    /// Returns the delay to wait before the next attempt, then doubles it.
    pub fn next_delay_ms(&mut self) -> u32 {
        let delay = self.current_ms;
        self.current_ms = self.current_ms.saturating_mul(2).min(self.max_ms);
        delay
    }

    /// Reset back to the initial delay after a successful connection.
    pub fn reset(&mut self) {
        self.current_ms = self.initial_ms;
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new()
    }
}
