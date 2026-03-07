use heizbox_core::event::DomainEvent;
use crate::{Button, InputEvent, ScreenType};
use super::{FrameBuffer, Navigation, Screen, ScreenError};

pub struct FireScreen {
    pub current_temp: u16,
    pub target_temp: u16,
    pub is_heating: bool,
}

impl FireScreen {
    pub fn new(target_temp: u16) -> Self {
        Self {
            current_temp: 0,
            target_temp,
            is_heating: false,
        }
    }
}

impl Screen for FireScreen {
    fn on_enter(&mut self) {
        self.is_heating = false;
    }

    fn on_exit(&mut self) {
        self.is_heating = false;
    }

    fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError> {
        match event.button {
            Button::Fire => {
                self.is_heating = !self.is_heating;
                Ok(Navigation::None)
            }
            Button::Center | Button::Up => Ok(Navigation::GoTo(ScreenType::Menu)),
            _ => Ok(Navigation::None),
        }
    }

    fn update(&mut self, event: DomainEvent) -> Result<(), ScreenError> {
        if let DomainEvent::TemperatureUpdated { current, .. } = event {
            self.current_temp = current;
        }
        Ok(())
    }

    fn render(&self) -> Result<FrameBuffer, ScreenError> {
        // Real rendering would push pixels to the TFT via the display driver.
        Ok(FrameBuffer::new(240, 280))
    }
}
