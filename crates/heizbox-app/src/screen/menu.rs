use heizbox_core::event::DomainEvent;
use crate::{Button, InputEvent, ScreenType};
use super::{FrameBuffer, Navigation, Screen, ScreenError};

pub struct MenuScreen {
    pub selected_index: usize,
}

impl MenuScreen {
<<<<<<< ours
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }
}

impl Default for MenuScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for MenuScreen {
    fn on_enter(&mut self) {
        self.selected_index = 0;
    }

    fn on_exit(&mut self) {}

    fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError> {
        match event.button {
            Button::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
                Ok(Navigation::None)
            }
            Button::Down => {
                self.selected_index = self.selected_index.saturating_add(1);
                Ok(Navigation::None)
            }
=======
    pub fn new() -> Self { Self { selected_index: 0 } }
}

impl Default for MenuScreen {
    fn default() -> Self { Self::new() }
}

impl Screen for MenuScreen {
    fn on_enter(&mut self) { self.selected_index = 0; }
    fn on_exit(&mut self)  {}

    fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError> {
        match event.button {
            Button::Up   => { self.selected_index = self.selected_index.saturating_sub(1); Ok(Navigation::None) }
            Button::Down => { self.selected_index = self.selected_index.saturating_add(1); Ok(Navigation::None) }
>>>>>>> theirs
            Button::Left | Button::Fire => Ok(Navigation::GoTo(ScreenType::Fire)),
            _ => Ok(Navigation::None),
        }
    }

<<<<<<< ours
    fn update(&mut self, _event: DomainEvent) -> Result<(), ScreenError> {
        Ok(())
    }
=======
    fn update(&mut self, _event: DomainEvent) -> Result<(), ScreenError> { Ok(()) }
>>>>>>> theirs

    fn render(&self) -> Result<FrameBuffer, ScreenError> {
        Ok(FrameBuffer::new(240, 280))
    }
}
