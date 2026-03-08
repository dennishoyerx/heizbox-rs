/// heizbox-tests — host-side test suite
/// All tests run on the host (x86/arm64), no embedded target required.

// TEST-T1: HeaterSm Idle→Heating with immediate cutoff.
// TEST-T2: ConsumptionData::record_cycle cumulative correctness.
// TEST-T3: DomainEvent serialises to camelCase JSON.
// TEST-T4: NavigationFsm rejects invalid transitions.
// TEST-T5: InputHandler Press vs LongPress classification.
// TEST-T6: HeaterConfigRepository returns default when NVS key missing.
// TEST-T7: ExponentialBackoff doubles delays up to maximum.
// TEST-T9: DomainEvent roundtrip serialize → deserialize.

// ── TEST-T1 ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test_heater_sm {
    use heizbox_core::heater::state::{HeaterSm, Idle, HeaterConfig};

    #[test]
    fn idle_to_heating_immediate_cutoff() {
        // TEST-T1 ✅
        let config = HeaterConfig::with_defaults();
        let sm: HeaterSm<Idle> = HeaterSm::new(config.clone());
        // Start heating at t=0 with no voltage tracking.
        let heating = sm.start_heating_with_voltage(0, None).unwrap();
        let cutoff = config.target_temp;
        // Feed a temperature above cutoff+20 → update_temperature returns Err.
        let result = heating.update_temperature(cutoff + 21, 100);
        assert!(result.is_err(), "expected CutoffTemperatureExceeded error");
    }
}

// ── TEST-T2 ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test_consumption {
    use heizbox_core::consumption::ConsumptionData;

    #[test]
    fn record_cycle_cumulates_correctly() {
        // TEST-T2 ✅
        let mut data = ConsumptionData::default();
        assert_eq!(data.total_cycles, 0);
        assert_eq!(data.total_duration_ms, 0);

        data.record_cycle(1_000);
        assert_eq!(data.total_cycles, 1);
        assert_eq!(data.total_duration_ms, 1_000);

        data.record_cycle(2_500);
        assert_eq!(data.total_cycles, 2);
        assert_eq!(data.total_duration_ms, 3_500);
    }

    #[test]
    fn record_cycle_no_double_count() {
        // Calling record_cycle twice with same params should count twice —
        // exactly-once is the caller's responsibility (documented in CORE-T6).
        let mut data = ConsumptionData::default();
        data.record_cycle(500);
        data.record_cycle(500);
        assert_eq!(data.total_cycles, 2, "each call should be counted once");
    }
}

// ── TEST-T3 ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test_domain_event_serialisation {
    use heizbox_core::event::DomainEvent;
    use serde_json;

    #[test]
    fn heating_started_is_camel_case() {
        // TEST-T3 ✅
        let event = DomainEvent::HeatingStarted { target_temp: 200 };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"targetTemp\""), "expected camelCase field, got: {}", json);
        assert!(json.contains("\"type\"") || json.contains("HeatingStarted"),
                "expected type discriminant in JSON");
    }

    #[test]
    fn session_update_fields_are_camel_case() {
        // TEST-T3 ✅  (CORE-T7 variant)
        let event = DomainEvent::SessionUpdate {
            clicks: 3,
            last_click: 1_700_000_000,
            session_start: 1_700_000_000 - 30_000,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"clicks\""),        "got: {}", json);
        assert!(json.contains("\"lastClick\""),     "got: {}", json);
        assert!(json.contains("\"sessionStart\""),  "got: {}", json);
    }

    #[test]
    fn heartbeat_sent_serialises() {
        // TEST-T3 ✅  (CORE-T8 variant)
        let event = DomainEvent::HeartbeatSent;
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.is_empty());
    }
}

// ── TEST-T4 ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test_navigation_fsm {
    use heizbox_app::screen::nav::NavigationFsm;
    use heizbox_app::ScreenType;

    #[test]
    fn invalid_transition_returns_error() {
        // TEST-T4 ✅
        let mut fsm = NavigationFsm::new();
        // Direct Startup → OtaUpdate is invalid (must go via Fire → Menu).
        let result = fsm.navigate_to(ScreenType::OtaUpdate);
        assert!(result.is_err(), "expected NavError for invalid transition");
    }

    #[test]
    fn valid_transition_startup_to_fire() {
        let mut fsm = NavigationFsm::new();
        let result = fsm.navigate_to(ScreenType::Fire);
        assert!(result.is_ok(), "Startup→Fire should be valid");
    }

    #[test]
    fn back_returns_to_previous_screen() {
        let mut fsm = NavigationFsm::new();
        fsm.navigate_to(ScreenType::Fire).unwrap();
        fsm.navigate_to(ScreenType::Menu).unwrap();
        let back = fsm.navigate_back();
        assert!(back.is_ok());
        assert_eq!(fsm.current(), ScreenType::Fire);
    }
}

// ── TEST-T5 ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test_input_handler {
    use heizbox_app::input::handler::InputHandler;
    use heizbox_app::{Button, InputEventType};

    #[test]
    fn short_press_is_press() {
        // TEST-T5 ✅
        let mut handler = InputHandler::new();
        // Simulate button down at t=0, up at t=100 ms (< LONG_PRESS_MS=300).
        handler.handle_input(Button::Center, true,  0);
        let event = handler.handle_input(Button::Center, false, 100);
        assert_eq!(event, Some(InputEventType::Press));
    }

    #[test]
    fn long_press_is_long_press() {
        // TEST-T5 ✅
        let mut handler = InputHandler::new();
        handler.handle_input(Button::Center, true,  0);
        let event = handler.handle_input(Button::Center, false, 400); // > 300 ms
        assert_eq!(event, Some(InputEventType::LongPress));
    }
}

// ── TEST-T6 ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test_heater_config_repo {
    use heizbox_hal::nvs::{NvsDriver, NvsError};
    use heizbox_infra::persistence::HeaterConfigRepository;

    struct MockNvs;

    impl NvsDriver for MockNvs {
        fn get_u8(&mut self, _ns: &str, key: &str) -> Result<Option<u8>, NvsError> {
            Err(NvsError::KeyNotFound(key.into()))
        }
        fn set_u8(&mut self, _ns: &str, _key: &str, _v: u8) -> Result<(), NvsError> { Ok(()) }
        fn get_u16(&mut self, _ns: &str, key: &str) -> Result<Option<u16>, NvsError> {
            Err(NvsError::KeyNotFound(key.into()))
        }
        fn set_u16(&mut self, _ns: &str, _key: &str, _v: u16) -> Result<(), NvsError> { Ok(()) }
        fn get_u32(&mut self, _ns: &str, key: &str) -> Result<Option<u32>, NvsError> {
            Err(NvsError::KeyNotFound(key.into()))
        }
        fn set_u32(&mut self, _ns: &str, _key: &str, _v: u32) -> Result<(), NvsError> { Ok(()) }
        fn get_blob(&mut self, _ns: &str, key: &str) -> Result<Option<heapless::Vec<u8, 64>>, NvsError> {
            Err(NvsError::KeyNotFound(key.into()))
        }
        fn set_blob(&mut self, _ns: &str, _key: &str, _data: &[u8]) -> Result<(), NvsError> { Ok(()) }
    }

    #[test]
    fn load_returns_defaults_when_nvs_empty() {
        // TEST-T6 ✅
        let mut repo = HeaterConfigRepository::new(MockNvs);
        let config = repo.load().expect("load should not fail with missing keys");
        assert_eq!(config.target_temp, heizbox_core::config::DEFAULT_TARGET_TEMP);
        assert_eq!(config.auto_stop_time_ms, heizbox_core::config::DEFAULT_AUTO_STOP_MS);
        assert_eq!(config.power_level, heizbox_core::config::DEFAULT_POWER);
    }
}

// ── TEST-T7 ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test_exponential_backoff {
    use heizbox_core::network::ExponentialBackoff;

    #[test]
    fn delays_double_up_to_maximum() {
        // TEST-T7 ✅
        let mut bo = ExponentialBackoff::new(500, 8_000);
        let d1 = bo.next_delay_ms();
        let d2 = bo.next_delay_ms();
        let d3 = bo.next_delay_ms();
        let d4 = bo.next_delay_ms();
        let d5 = bo.next_delay_ms();

        assert_eq!(d1, 500);
        assert_eq!(d2, 1_000);
        assert_eq!(d3, 2_000);
        assert_eq!(d4, 4_000);
        assert_eq!(d5, 8_000, "should cap at maximum");
        // Further calls should stay at maximum.
        assert_eq!(bo.next_delay_ms(), 8_000);
    }

    #[test]
    fn reset_restarts_from_initial() {
        let mut bo = ExponentialBackoff::new(1_000, 16_000);
        bo.next_delay_ms();
        bo.next_delay_ms();
        bo.reset();
        assert_eq!(bo.next_delay_ms(), 1_000);
    }
}

// ── TEST-T9 ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test_domain_event_roundtrip {
    use heizbox_core::event::DomainEvent;
    use serde_json;

    #[test]
    fn heat_cycle_completed_roundtrip() {
        // TEST-T9 ✅
        let original = DomainEvent::HeatCycleCompleted {
            duration_ms: 45_000,
            cycle: 7,
        };
        let json = serde_json::to_string(&original).unwrap();
        let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
        // Compare via re-serialisation (DomainEvent may not impl PartialEq).
        let json2 = serde_json::to_string(&decoded).unwrap();
        assert_eq!(json, json2, "roundtrip should produce identical JSON");
    }

    #[test]
    fn status_update_roundtrip() {
        // TEST-T9 ✅
        let original = DomainEvent::StatusUpdate { is_on: true, is_heating: false };
        let json = serde_json::to_string(&original).unwrap();
        let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&decoded).unwrap();
        assert_eq!(json, json2);
    }
}
