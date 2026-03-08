use heizbox_core::event::DomainEvent;
use crate::{InputEvent, ScreenType};
use super::{FrameBuffer, Navigation, Screen, ScreenError};

/// APP-T6: OTA progress screen.
pub struct OtaUpdateScreen {
    pub progress_percent: u8,
}

impl OtaUpdateScreen {
    pub fn new() -> Self { Self { progress_percent: 0 } }
}

impl Default for OtaUpdateScreen {
    fn default() -> Self { Self::new() }
}

impl Screen for OtaUpdateScreen {
    fn on_enter(&mut self) { self.progress_percent = 0; }
    fn on_exit(&mut self)  {}

    fn handle_input(&mut self, _event: InputEvent) -> Result<Navigation, ScreenError> {
        Ok(Navigation::None) // No user input during OTA
    }

    fn update(&mut self, event: DomainEvent) -> Result<(), ScreenError> {
        if let DomainEvent::OtaProgress { percent } = event {
            self.progress_percent = percent;
        }
        if let DomainEvent::OtaCompleted = event {
            return Ok(()); // Reboot handled externally
        }
        Ok(())
    }

    fn render(&self) -> Result<FrameBuffer, ScreenError> {
        Ok(FrameBuffer::new(240, 280))
    }
}
