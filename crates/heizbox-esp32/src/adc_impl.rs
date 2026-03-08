/// ADC implementation for battery voltage monitoring.
///
/// ESP32-T18: AdcImpl using esp_idf_hal::adc::AdcDriver.
/// ESP32-T19: Voltage divider calibration (12-bit ADC, 3.3V ref, divider ratio).

use heizbox_hal::adc::{AdcDriver, AdcError};

/// Voltage divider ratio for battery pin.
/// Typical: 100k/(100k+100k) = 0.5 → ratio = 2.0
const DIVIDER_RATIO: f32 = 2.0;
const ADC_REF_V: f32    = 3.3;
const ADC_MAX: f32      = 4095.0;

pub struct AdcImpl;

impl AdcImpl {
    pub fn new() -> Self { Self }

    /// Convert raw ADC reading to battery voltage in millivolts.
    /// ESP32-T19 ✅
    pub fn raw_to_mv(raw: u16) -> u32 {
        let v_adc = (raw as f32 / ADC_MAX) * ADC_REF_V;
        ((v_adc * DIVIDER_RATIO) * 1000.0) as u32
    }
}

impl AdcDriver for AdcImpl {
    fn read(&self, pin: u8) -> Result<u16, AdcError> {
        #[cfg(target_os = "espidf")]
        {
            log::debug!("AdcImpl: read pin {}", pin);
            Ok(2048) // placeholder; real integration via stored AtcChannelDriver
        }
        #[cfg(not(target_os = "espidf"))]
        Ok(2048)
    }
}
