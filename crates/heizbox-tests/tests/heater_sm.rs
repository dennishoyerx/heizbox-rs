#[cfg(test)]
mod heater_sm_tests {
    use heizbox_core::heater::*;

    fn default_sm() -> HeaterSm<Idle> {
        HeaterSm::<Idle>::new(HeaterConfig {
            power: 100,
            target_temp: 200,
            auto_stop_time_ms: 90_000,
        })
    }

    #[test]
    fn idle_to_heating() {
        let sm = default_sm();
        let heating = sm.start_heating(1_000).expect("should transition to Heating");
        assert_eq!(heating.state.cycle_started_at, 1_000);
    }

    #[test]
    fn cutoff_exceeded_returns_error() {
        let sm = default_sm()
            .start_heating(0)
            .unwrap();

        // 225 °C > 200 + 20 — should trigger cutoff
        let result = sm.update_temperature(225, 1_000);
        assert!(matches!(result, Err(HeaterError::CutoffTemperatureExceeded)));
    }

    #[test]
    fn timeout_exceeded_returns_error() {
        let sm = default_sm()
            .start_heating(0)
            .unwrap();

        // now_ms > auto_stop_time_ms (90 000)
        let result = sm.update_temperature(150, 91_000);
        assert!(matches!(result, Err(HeaterError::CycleTimeoutExceeded)));
    }

    #[test]
    fn normal_update_succeeds() {
        let sm = default_sm()
            .start_heating(0)
            .unwrap()
            .update_temperature(150, 1_000)
            .unwrap();

        assert_eq!(sm.current_temp, 150);
        assert_eq!(sm.cycle_duration_ms, 1_000);
    }

    #[test]
    fn heating_to_paused() {
        let _paused: HeaterSm<Paused> = default_sm()
            .start_heating(0)
            .unwrap()
            .update_temperature(150, 1_000)
            .unwrap()
            .pause();
    }

    #[test]
    fn paused_to_heating_resume() {
        let heating: HeaterSm<Heating> = default_sm()
            .start_heating(0)
            .unwrap()
            .update_temperature(150, 1_000)
            .unwrap()
            .pause()
            .resume(2_000);

        // Resuming at 2 000 ms with 1 000 ms already elapsed means
        // adjusted start = 2 000 − 1 000 = 1 000.
        assert_eq!(heating.state.cycle_started_at, 1_000);
    }

    #[test]
    fn paused_to_idle_finalize() {
        let (_idle, result) = default_sm()
            .start_heating(0)
            .unwrap()
            .update_temperature(180, 5_000)
            .unwrap()
            .pause()
            .finalize();

        assert_eq!(result.max_temp, 180);
        assert_eq!(result.duration_ms, 5_000);
        assert_eq!(result.started_at, Some(0));
    }

    #[test]
    fn target_reached_flag() {
        let sm = default_sm()
            .start_heating(0)
            .unwrap()
            .update_temperature(200, 1_000)
            .unwrap();

        assert!(sm.is_target_reached());
    }
}
