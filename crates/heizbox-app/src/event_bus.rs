//! Event bus — will be backed by FreeRTOS queues in the ESP32 crate.
//! Embassy channels are commented out until the runtime is wired in.

use heizbox_core::event::DomainEvent;

// Placeholder no-op bus for compilation.
pub struct EventBus;

impl EventBus {
    pub fn new() -> Self {
        Self
    }

    /// Publish an event (no-op stub).
    pub fn publish(&mut self, _event: DomainEvent) {}
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
