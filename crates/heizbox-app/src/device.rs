use heizbox_core::event::DomainEvent;
use heizbox_core::input::InputEvent as CoreInputEvent;
use heizbox_core::error::SensorError;
use heizbox_hal::sensors::mlx90614::Mlx90614;

/// Top-level application struct. Owns all managers and drives the event loop.
/// Concrete initialisation happens in `heizbox-esp32`.
pub struct DeviceApp {
    /// Pending domain events waiting to be dispatched.
    pending_events: heapless::Vec<DomainEvent, 16>,
    /// Optional IR temperature sensor.
    mlx90614: Option<Mlx90614>,
}

impl DeviceApp {
    pub fn new() -> Self {
        Self {
            pending_events: heapless::Vec::new(),
            mlx90614: None,
        }
    }

    /// Create with a MLX90614 sensor already attached.
    pub fn with_sensor(mlx90614: Mlx90614) -> Self {
        Self {
            pending_events: heapless::Vec::new(),
            mlx90614: Some(mlx90614),
        }
    }

    /// Called from the control task every ~100 ms.
    pub fn update_heater(&mut self) {
        // Placeholder — heater SM tick goes here.
    }

    /// Called from the control task after `update_heater`.
    /// Reads MLX90614 temperatures and publishes TemperatureUpdated event.
    pub fn update_sensors(&mut self) {
        if let Some(sensor) = &mut self.mlx90614 {
            match sensor.read_all() {
                Ok((object_c, ambient_c, raw_ir)) => {
                    let event = DomainEvent::TemperatureUpdated {
                        current: object_c,
                        ambient: ambient_c,
                        raw_ir,
                    };
                    let _ = self.push_event(event);
                }
                Err(_e) => {
                    // Convert I2cError to SensorError if needed.
                    let _sensor_err = SensorError::I2cFailed;
                    // Could push an error event if desired.
                    // For now, ignore and continue.
                }
            }
        }
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
