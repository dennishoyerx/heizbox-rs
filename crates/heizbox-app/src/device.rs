use heizbox_core::event::DomainEvent;
use heizbox_core::input::InputEvent as CoreInputEvent;
use heizbox_core::error::SensorError;
use heizbox_hal::sensors::mlx90614::Mlx90614;
use crate::screen::FrameBuffer;
use heizbox_core::heater::{HeaterSm, HeaterConfig, HeaterError, CycleResult, Idle, Heating};
use heizbox_core::consumption::ConsumptionData;
use crate::event_bus::EventBus;
use heizbox_core::event::HeaterErrorEvent;

/// Top-level application struct. Owns all managers and drives the event loop.
/// Concrete initialisation happens in `heizbox-esp32`.
pub struct DeviceApp {
    /// Pending domain events waiting to be dispatched.
    pending_events: heapless::Vec<DomainEvent, 16>,
    /// Optional IR temperature sensor.
    mlx90614: Option<Mlx90614>,
    /// State machine for heater control (active when heating).
    heater_sm: Option<HeaterSm<Heating>>,
    /// Event bus for inter-task communication.
    event_bus: EventBus,
    /// Consumption statistics.
    consumption: ConsumptionData,
}

impl DeviceApp {
    pub fn new() -> Self {
        Self {
            pending_events: heapless::Vec::new(),
            mlx90614: None,
            heater_sm: None,
            event_bus: EventBus::new(),
            consumption: ConsumptionData::new(),
        }
    }

    /// Create with a MLX90614 sensor already attached.
    pub fn with_sensor(mlx90614: Mlx90614) -> Self {
        Self {
            pending_events: heapless::Vec::new(),
            mlx90614: Some(mlx90614),
            heater_sm: None,
            event_bus: EventBus::new(),
            consumption: ConsumptionData::new(),
        }
    }

    /// Called from the control task with current timestamp (ms since boot).
    /// Performs sensor reading, heater state update, event generation, and consumption tracking.
    pub fn update_heater(&mut self, now_ms: u32) {
        // Read sensor values
        let temp_result = match &mut self.mlx90614 {
            Some(sensor) => sensor.read_all(),
            None => {
                // No sensor configured, nothing to do
                return;
            }
        };

        match temp_result {
            Ok((object_c, ambient_c, raw_ir)) => {
                // Publish temperature update event
                self.push_event(DomainEvent::TemperatureUpdated {
                    current: object_c,
                    ambient: ambient_c,
                    raw_ir,
                });

                // If no active heating, nothing more to do
                if self.heater_sm.is_none() {
                    return;
                }

                // Take ownership of the heating state machine
                let heating_opt = self.heater_sm.take();
                if let Some(mut heating) = heating_opt {
                    // Capture needed data before update for potential error reporting
                    let target_temp = heating.target_temp;
                    let auto_stop = heating.auto_stop_time_ms;
                    let cycle_started_at = heating.cycle_started_at;

                    match heating.update_temperature(object_c, now_ms) {
                        Ok(updated) => {
                            if updated.is_target_reached() {
                                // Finalize cycle
                                let paused = updated.pause();
                                let (idle, cycle_result) = paused.finalize();
                                self.consumption.record_cycle(cycle_result.duration_ms);
                                self.push_event(DomainEvent::CycleFinished(cycle_result));
                                // heater_sm remains None (idle)
                            } else {
                                // Continue heating
                                self.heater_sm = Some(updated);
                            }
                        }
                        Err(e) => {
                            // Construct error event using captured data and current reading
                            let err_event = match e {
                                HeaterError::CutoffTemperatureExceeded => {
                                    let limit = target_temp + 20;
                                    DomainEvent::HeatingError(HeaterErrorEvent::CutoffExceeded {
                                        temp: object_c,
                                        limit,
                                    })
                                }
                                HeaterError::CycleTimeoutExceeded => {
                                    let duration = now_ms - cycle_started_at;
                                    DomainEvent::HeatingError(HeaterErrorEvent::TimeoutExceeded {
                                        duration,
                                        limit: auto_stop,
                                    })
                                }
                                HeaterError::InvalidTemperatureReading => {
                                    DomainEvent::HeatingError(HeaterErrorEvent::InvalidReading {
                                        reason: "Invalid temperature reading",
                                    })
                                }
                                HeaterError::CalibrationFailed => {
                                    DomainEvent::HeatingError(HeaterErrorEvent::InvalidReading {
                                        reason: "Calibration failed",
                                    })
                                }
                            };
                            self.push_event(err_event);
                            // Heating stopped; heater_sm stays None
                        }
                    }
                }
            }
            Err(_e) => {
                // Sensor read error
                self.push_event(DomainEvent::HeatingError(HeaterErrorEvent::InvalidReading {
                    reason: "Sensor read failed",
                }));
                // Stop heating if active (sensor failure is critical)
                if self.heater_sm.is_some() {
                    self.heater_sm = None;
                }
            }
        }
    }

    /// Start a new heating cycle. Returns error if already heating.
    pub fn start_heating(&mut self, now_ms: u32) -> Result<(), HeaterError> {
        if self.heater_sm.is_none() {
            let sm_idle = HeaterSm::<Idle>::new(HeaterConfig::with_defaults());
            let sm_heating = sm_idle.start_heating(now_ms)?;
            let target_temp = sm_heating.target_temp;
            self.heater_sm = Some(sm_heating);
            self.push_event(DomainEvent::HeatingStarted {
                target_temp,
                timestamp_ms: now_ms,
            });
            Ok(())
        } else {
            Err(HeaterError::InvalidTemperatureReading)
        }
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
        self.event_bus.publish(event);
    }

    /// Handle a physical input event.
    pub fn handle_input(&mut self, _event: CoreInputEvent) {
        // Placeholder — forward to active screen.
    }

    /// Render the active screen to the display.
    /// Returns the framebuffer that was produced.
    /// Currently returns a black screen (placeholder).
    pub fn render(&mut self) -> FrameBuffer {
        // Placeholder — in the future, this will render the active screen.
        // For now, create a black framebuffer (all zeros).
        FrameBuffer::new(240, 280)
    }
}

impl Default for DeviceApp {
    fn default() -> Self {
        Self::new()
    }
}
