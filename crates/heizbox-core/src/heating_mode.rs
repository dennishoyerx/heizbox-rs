//! CORE-T4: `HeatingMode` defined at domain level so it can be shared between
//! `heizbox-app` (screen UI) and `heizbox-infra` (persistence) without
//! creating a cross-dependency.

use serde::{Deserialize, Serialize};
use crate::config::*;

/// Operating mode for a single heating cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HeatingMode {
    /// User-selectable temperature preset.
    #[default]
    Preset,
    /// Continuous free-form temperature target (set manually).
    Temperature,
}

/// A named temperature preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    /// Low temperature; emphasises flavour / terpenes.
    Flavor,
    /// Mid-range all-rounder.
    Balanced,
    /// Higher temperature for fuller extraction.
    Extraction,
    /// Maximum temperature; full extraction, DynaVap CUTOFF_DISABLED path.
    Full,
}

/// CORE-T5: Map each preset to its target temperature.
impl Preset {
    /// Target temperature for this preset (°C).
    pub const fn target_temp(self) -> u16 {
        match self {
            Self::Flavor     => PRESET_FLAVOR,
            Self::Balanced   => PRESET_BALANCED,
            Self::Extraction => PRESET_EXTRACTION,
            Self::Full       => PRESET_FULL,
        }
    }

    /// Power level (0–100 %) recommended for this preset.
    pub const fn power(self) -> u8 {
        match self {
            Self::Flavor     => 80,
            Self::Balanced   => 90,
            Self::Extraction => 95,
            Self::Full       => 100,
        }
    }

    /// All four presets in order.
    pub const ALL: [Preset; 4] = [
        Self::Flavor,
        Self::Balanced,
        Self::Extraction,
        Self::Full,
    ];
}
