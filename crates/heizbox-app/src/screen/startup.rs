use async_trait::async_trait;
use heizbox_core::event::DomainEvent;
use crate::{InputEvent, ScreenType};
use super::{FrameBuffer, Navigation, Screen, ScreenError};

pub struct StartupScreen;

#[async_trait]
impl Screen for StartupScreen {
    async fn on_enter(&mut self) {}
    async fn on_exit(&mut self) {}

    async fn handle_input(&mut self, _event: InputEvent) -> Result<Navigation, ScreenError> {
        Ok(Navigation::GoTo(ScreenType::Fire))
    }

    async fn update(&mut self, _event: DomainEvent) -> Result<(), ScreenError> {
        Ok(())
    }

    async fn render(&self) -> Result<FrameBuffer, ScreenError> {
        Ok(FrameBuffer::new(280, 240))
    }
}
