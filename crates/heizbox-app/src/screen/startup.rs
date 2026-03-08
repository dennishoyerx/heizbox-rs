use heizbox_core::event::DomainEvent;
use crate::{InputEvent, ScreenType};
use super::{FrameBuffer, Navigation, Screen, ScreenError};

pub struct StartupScreen;

impl Screen for StartupScreen {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self)  {}

    fn handle_input(&mut self, _event: InputEvent) -> Result<Navigation, ScreenError> {
        Ok(Navigation::GoTo(ScreenType::Fire))
    }

    fn update(&mut self, _event: DomainEvent) -> Result<(), ScreenError> { Ok(()) }

    fn render(&self) -> Result<FrameBuffer, ScreenError> {
        Ok(FrameBuffer::new(240, 280))
    }
}
