use heizbox_core::event::DomainEvent;
use heizbox_core::input::InputEvent as CoreInputEvent;
use heizbox_core::error::SensorError;
use heizbox_hal::sensors::mlx90614::Mlx90614;
use crate::screen::FrameBuffer;
use heizbox_core::heater::{HeaterSm, HeaterConfig, HeaterError, Idle, Heating};
use heizbox_core::consumption::ConsumptionData;
use crate::event_bus::EventBus;
use heizbox_core::event::HeaterErrorEvent;

/// Top-level application struct (APP-T12 / APP-T13).
/// Owns all managers and drives the event loop.
pub struct DeviceApp {
    pending_events: heapless::Vec<DomainEvent, 16>,
    mlx90614:       Option<Mlx90614>,
    heater_sm:      Option<HeaterSm<Heating>>,
    event_bus:      EventBus,
    consumption:    ConsumptionData,
}

impl DeviceApp {
    pub fn new() -> Self {
        Self {
            pending_events: heapless::Vec::new(),
            mlx90614:       None,
            heater_sm:      None,
            event_bus:      EventBus::new(),
            consumption:    ConsumptionData::new(),
        }
    }

    pub fn with_sensor(mlx90614: Mlx90614) -> Self {
        Self {
            pending_events: heapless::Vec::new(),
            mlx90614:       Some(mlx90614),
            heater_sm:      None,
            event_bus:      EventBus::new(),
            consumption:    ConsumptionData::new(),
        }
    }

    // ── APP-T12: Control tick ─────────────────────────────────────────────

    /// Called from the control task (≈100 ms interval).
    /// Reads sensor → updates HeaterSm → generates DomainEvents → publishes to EventBus.
    pub fn update_heater(&mut self, now_ms: u32) {
        let temp_result = match &mut self.mlx90614 {
            Some(sensor) => sensor.read_all(),
            None         => return,
        };

        match temp_result {
            Ok((object_c, ambient_c, raw_ir)) => {
                self.push_event(DomainEvent::TemperatureUpdated {
                    current: object_c,
                    ambient: ambient_c,
                    raw_ir,
                });

                if self.heater_sm.is_none() {
                    return;
                }

                let heating = self.heater_sm.take().unwrap();
                let target_temp       = heating.target_temp;
                let auto_stop         = heating.auto_stop_time_ms;
                let cycle_started_at  = heating.state.cycle_started_at;

                match heating.update_temperature(object_c, now_ms) {
                    Ok(updated) => {
                        if updated.is_target_reached() {
                            let paused = updated.pause();
                            let (_, cycle_result) = paused.finalize();
                            self.consumption.record_cycle(cycle_result.duration_ms);
                            self.push_event(DomainEvent::CycleFinished(cycle_result));
                        } else {
                            self.heater_sm = Some(updated);
                        }
                    }
                    Err(e) => {
                        let err_event = match e {
                            HeaterError::CutoffTemperatureExceeded => {
                                DomainEvent::HeatingError(HeaterErrorEvent::CutoffExceeded {
                                    temp:  object_c,
                                    limit: target_temp + 20,
                                })
                            }
                            HeaterError::CycleTimeoutExceeded => {
                                DomainEvent::HeatingError(HeaterErrorEvent::TimeoutExceeded {
                                    duration: now_ms - cycle_started_at,
                                    limit:    auto_stop,
                                })
                            }
                            _ => DomainEvent::HeatingError(HeaterErrorEvent::InvalidReading {
                                reason: "Heater error",
                            }),
                        };
                        self.push_event(err_event);
                    }
                }
            }
            Err(_) => {
                self.push_event(DomainEvent::HeatingError(HeaterErrorEvent::InvalidReading {
                    reason: "Sensor read failed",
                }));
                self.heater_sm = None;
            }
        }
    }

    pub fn start_heating(&mut self, now_ms: u32) -> Result<(), HeaterError> {
        if self.heater_sm.is_none() {
            let sm     = HeaterSm::<Idle>::new(HeaterConfig::with_defaults());
            let heated = sm.start_heating(now_ms)?;
            let target = heated.target_temp;
            self.heater_sm = Some(heated);
            self.push_event(DomainEvent::HeatingStarted {
                target_temp:  target,
                timestamp_ms: now_ms,
            });
            Ok(())
        } else {
            Err(HeaterError::InvalidTemperatureReading)
        }
    }

    pub fn update_sensors(&mut self) {
        if let Some(sensor) = &mut self.mlx90614 {
            if let Ok((object_c, ambient_c, raw_ir)) = sensor.read_all() {
                let event = DomainEvent::TemperatureUpdated { current: object_c, ambient: ambient_c, raw_ir };
                let _ = self.push_event(event);
            }
        }
    }

    // ── APP-T13: UI tick ──────────────────────────────────────────────────

    /// Called from the ui task (≈50 ms / 20 fps).
    /// Returns a black framebuffer (stub — real rendering goes here).
    pub fn render(&mut self) -> FrameBuffer {
        FrameBuffer::new(240, 280)
    }

    // ── Event queue ───────────────────────────────────────────────────────

    pub fn pop_event(&mut self) -> Option<DomainEvent> {
        if self.pending_events.is_empty() {
            None
        } else {
            Some(self.pending_events.remove(0))
        }
    }

    pub fn push_event(&mut self, event: DomainEvent) {
        let _ = self.pending_events.push(event.clone());
        self.event_bus.publish(event);
    }

    pub fn handle_input(&mut self, _event: CoreInputEvent) {}
}

impl Default for DeviceApp {
    fn default() -> Self { Self::new() }
}
