use heizbox_core::event::DomainEvent;
use heizbox_core::input::InputEvent as CoreInputEvent;

/// Top-level application struct. Owns all managers and drives the event loop.
/// Concrete initialisation happens in `heizbox-esp32`.
pub struct DeviceApp {
    /// Pending domain events waiting to be dispatched.
    pending_events: heapless::Vec<DomainEvent, 16>,
}

impl DeviceApp {
    pub fn new() -> Self {
        Self {
            pending_events: heapless::Vec::new(),
        }
    }

    /// Called from the control task every ~100 ms.
    pub fn update_heater(&mut self) {
        // Placeholder — heater SM tick goes here.
    }

    /// Called from the control task after `update_heater`.
    pub fn update_sensors(&mut self) {
        // Placeholder — read IR/MLX90614 temperature here.
    }

    /// Drain the first pending event, if any.
    pub fn pop_event(&mut self) -> Option<DomainEvent> {
        if self.pending_events.is_empty() {
            None
        } else {
            Some(self.pending_events.remove(0))
        }
    }

    /// Push a domain event onto the internal queue.
    pub fn push_event(&mut self, event: DomainEvent) {
        let _ = self.pending_events.push(event);
    }

    /// Handle a physical input event.
    pub fn handle_input(&mut self, _event: CoreInputEvent) {
        // Placeholder — forward to active screen.
    }

    /// Render the active screen to the display.
    pub fn render(&mut self) {
        // Placeholder — call active Screen::render().
    }
}

impl Default for DeviceApp {
    fn default() -> Self {
        Self::new()
    }
}
