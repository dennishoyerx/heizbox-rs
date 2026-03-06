use async_trait::async_trait;
use heizbox_core::event::DomainEvent;
use crate::{Button, InputEvent, ScreenType};
use super::{FrameBuffer, Navigation, Screen, ScreenError};

pub struct MenuScreen {
    pub selected_index: usize,
}

impl MenuScreen {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }
}

impl Default for MenuScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Screen for MenuScreen {
    async fn on_enter(&mut self) {
        self.selected_index = 0;
    }

    async fn on_exit(&mut self) {}

    async fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError> {
        match event.button {
            Button::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
                Ok(Navigation::None)
            }
            Button::Down => {
                self.selected_index = self.selected_index.saturating_add(1);
                Ok(Navigation::None)
            }
            Button::Left | Button::Fire => Ok(Navigation::GoTo(ScreenType::Fire)),
            _ => Ok(Navigation::None),
        }
    }

    async fn update(&mut self, _event: DomainEvent) -> Result<(), ScreenError> {
        Ok(())
    }

    async fn render(&self) -> Result<FrameBuffer, ScreenError> {
        Ok(FrameBuffer::new(280, 240))
    }
}
