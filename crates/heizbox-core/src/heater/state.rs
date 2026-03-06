use serde::{Deserialize, Serialize};

// ── State marker types ────────────────────────────────────────────────────────

pub struct Idle;

pub struct Heating {
    pub cycle_started_at: u32,
}

pub struct Paused {
    pub cycle_started_at: u32,
    pub elapsed_ms: u32,
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

    /// Idle → Heating
    pub fn start_heating(self, cycle_started_at: u32) -> Result<HeaterSm<Heating>, HeaterError> {
        Ok(HeaterSm {
            power: self.power,
            target_temp: self.target_temp,
            current_temp: self.current_temp,
            auto_stop_time_ms: self.auto_stop_time_ms,
            cycle_duration_ms: 0,
            ir_calibration: self.ir_calibration,
            state: Heating { cycle_started_at },
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
            },
        }
    }

    fn is_cutoff_exceeded(&self) -> bool {
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
            },
        }
    }

    /// Paused → Idle, returns the finished cycle result.
    pub fn finalize(self) -> (HeaterSm<Idle>, CycleResult) {
        let result = CycleResult {
            duration_ms: self.cycle_duration_ms,
            max_temp: self.current_temp,
            started_at: Some(self.state.cycle_started_at),
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
    pub fn with_defaults() -> Self {
        Self {
            power: crate::config::DEFAULT_POWER,
            target_temp: crate::config::DEFAULT_TARGET_TEMP,
            auto_stop_time_ms: crate::config::DEFAULT_AUTO_STOP_MS,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CycleResult {
    pub duration_ms: u32,
    pub max_temp: u16,
    pub started_at: Option<u32>,
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
