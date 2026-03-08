use serde::{Deserialize, Serialize};

// ── State marker types ────────────────────────────────────────────────────────

pub struct Idle;

pub struct Heating {
    pub cycle_started_at: u32,
    /// Battery voltage sampled at the moment heating started (V).
    /// Populated by `start_heating_with_voltage`; `None` when ADC is unavailable.
    pub voltage_start: Option<f32>,
}

pub struct Paused {
    pub cycle_started_at: u32,
    pub elapsed_ms: u32,
    pub voltage_start: Option<f32>,
}

// ── State machine ─────────────────────────────────────────────────────────────

pub struct HeaterSm<S> {
    pub power: u8,
    pub target_temp: u16,
    pub current_temp: u16,
    pub auto_stop_time_ms: u32,
    pub cycle_duration_ms: u32,
    pub ir_calibration: IrCalibration,
    pub state: S,
}

impl HeaterSm<Idle> {
    pub fn new(config: HeaterConfig) -> Self {
        Self {
            power: config.power,
            target_temp: config.target_temp,
            current_temp: 0,
            auto_stop_time_ms: config.auto_stop_time_ms,
            cycle_duration_ms: 0,
            ir_calibration: IrCalibration::default(),
            state: Idle,
        }
    }

    /// Idle → Heating (no voltage tracking).
    pub fn start_heating(self, cycle_started_at: u32) -> Result<HeaterSm<Heating>, HeaterError> {
        self.start_heating_with_voltage(cycle_started_at, None)
    }

    /// Idle → Heating, optionally capturing the battery voltage at start (CORE-T2).
    pub fn start_heating_with_voltage(
        self,
        cycle_started_at: u32,
        voltage_start: Option<f32>,
    ) -> Result<HeaterSm<Heating>, HeaterError> {
        Ok(HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: 0,
            ir_calibration: self.ir_calibration,
            state: Heating { cycle_started_at, voltage_start },
        })
    }
}

impl HeaterSm<Heating> {
    /// Update temperature and run safety checks.
    pub fn update_temperature(
        mut self,
        new_temp: u16,
        now_ms: u32,
    ) -> Result<HeaterSm<Heating>, HeaterError> {
        self.current_temp = new_temp;
        self.cycle_duration_ms = now_ms.saturating_sub(self.state.cycle_started_at);

        if self.is_cutoff_exceeded() {
            return Err(HeaterError::CutoffTemperatureExceeded);
        }
        if self.is_timeout_exceeded() {
            return Err(HeaterError::CycleTimeoutExceeded);
        }

        Ok(self)
    }

    pub fn is_target_reached(&self) -> bool {
        self.current_temp >= self.target_temp
    }

    /// Heating → Paused
    pub fn pause(self) -> HeaterSm<Paused> {
        HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: self.cycle_duration_ms,
            ir_calibration: self.ir_calibration,
            state: Paused {
                cycle_started_at: self.state.cycle_started_at,
                elapsed_ms: self.cycle_duration_ms,
                voltage_start: self.state.voltage_start,
            },
        }
    }

    fn is_cutoff_exceeded(&self) -> bool {
        // CORE-T3: `target_temp == CUTOFF_DISABLED (420)` disables the check entirely.
        if self.target_temp == crate::config::CUTOFF_DISABLED {
            return false;
        }
        self.current_temp > self.target_temp + 20
    }

    fn is_timeout_exceeded(&self) -> bool {
        self.cycle_duration_ms > self.auto_stop_time_ms
    }
}

impl HeaterSm<Paused> {
    /// Paused → Heating (resumes, adjusting start time for elapsed duration).
    pub fn resume(self, now_ms: u32) -> HeaterSm<Heating> {
        let adjusted_start = now_ms.saturating_sub(self.state.elapsed_ms);
        HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: self.cycle_duration_ms,
            ir_calibration: self.ir_calibration,
            state: Heating {
                cycle_started_at: adjusted_start,
                voltage_start: self.state.voltage_start,
            },
        }
    }

    /// Paused → Idle, returns the finished cycle result.
    /// Pass `voltage_end` from ADC for full battery-tracking (CORE-T2).
    pub fn finalize(self) -> (HeaterSm<Idle>, CycleResult) {
        self.finalize_with_voltage(None)
    }

    /// Paused → Idle, with explicit end-voltage (CORE-T2).
    pub fn finalize_with_voltage(
        self,
        voltage_end: Option<f32>,
    ) -> (HeaterSm<Idle>, CycleResult) {
        let result = CycleResult {
            duration_ms: self.cycle_duration_ms,
            max_temp: self.current_temp,
            started_at: Some(self.state.cycle_started_at),
            voltage_start: self.state.voltage_start,  // CORE-T2
            voltage_end,                               // CORE-T2
        };
        let idle = HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: 0,
            ir_calibration: self.ir_calibration,
            state: Idle,
        };
        (idle, result)
    }
}

// ── Supporting types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct IrCalibration {
    pub emissivity: u8,
    pub correction_offset: i8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HeaterConfig {
    pub power: u8,
    pub target_temp: u16,
    pub auto_stop_time_ms: u32,
}

impl HeaterConfig {
    /// Returns a `HeaterConfig` with device-validated default values (CORE-T1).
    pub fn with_defaults() -> Self {
        Self {
            power: crate::config::DEFAULT_POWER,
            target_temp: crate::config::DEFAULT_TARGET_TEMP,
            auto_stop_time_ms: crate::config::DEFAULT_AUTO_STOP_MS,
        }
    }
}

/// Result of a completed heating cycle — includes battery voltage data (CORE-T2).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CycleResult {
    /// Heating duration in milliseconds.
    pub duration_ms: u32,
    /// Highest measured temperature during this cycle (°C).
    pub max_temp: u16,
    /// Boot-relative timestamp when heating started (ms since boot).
    pub started_at: Option<u32>,
    /// Battery voltage when heating started (V).  `None` if ADC unavailable.
    pub voltage_start: Option<f32>,
    /// Battery voltage when heating ended (V).  `None` if ADC unavailable.
    pub voltage_end: Option<f32>,
}

#[derive(Debug)]
pub enum HeaterError {
    CutoffTemperatureExceeded,
    CycleTimeoutExceeded,
    InvalidTemperatureReading,
    CalibrationFailed,
}

impl core::fmt::Display for HeaterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CutoffTemperatureExceeded => write!(f, "Cutoff temperature exceeded"),
            Self::CycleTimeoutExceeded => write!(f, "Cycle timeout exceeded"),
            Self::InvalidTemperatureReading => write!(f, "Invalid temperature reading"),
            Self::CalibrationFailed => write!(f, "Calibration failed"),
        }
    }
}
