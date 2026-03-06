use crate::{Button, InputEvent, InputEventType};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Unknown input error")]
    Unknown,
}

#[derive(Clone, Copy)]
enum HandlerState {
    Normal,
    LongPressActive,
    MenuMode,
}

#[derive(Clone, Copy, Default)]
struct ButtonState {
    pressed_at: u32,
    is_pressed: bool,
}

pub struct InputHandler {
    state: HandlerState,
    button_states: [ButtonState; 6],
    last_input_ms: u32,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            state: HandlerState::Normal,
            button_states: [ButtonState::default(); 6],
            last_input_ms: 0,
        }
    }

    pub fn handle_input(
        &mut self,
        button: Button,
        now_ms: u32,
        is_pressed: bool,
    ) -> Result<Option<InputEvent>, InputError> {
        self.last_input_ms = now_ms;
        match self.state {
            HandlerState::Normal => self.handle_normal(button, now_ms, is_pressed),
            HandlerState::LongPressActive => self.handle_longpress(button, is_pressed),
            HandlerState::MenuMode => self.handle_menu(button, is_pressed),
        }
    }

    fn handle_normal(
        &mut self,
        button: Button,
        now_ms: u32,
        is_pressed: bool,
    ) -> Result<Option<InputEvent>, InputError> {
        let idx = button as usize;
        if is_pressed {
            self.button_states[idx].pressed_at = now_ms;
            self.button_states[idx].is_pressed = true;
            Ok(None)
        } else {
            let hold_ms = now_ms.saturating_sub(self.button_states[idx].pressed_at);
            self.button_states[idx].is_pressed = false;
            if hold_ms > 300 {
                self.state = HandlerState::LongPressActive;
                Ok(Some(InputEvent { button, event_type: InputEventType::LongPress }))
            } else {
                Ok(Some(InputEvent { button, event_type: InputEventType::Press }))
            }
        }
    }

    fn handle_longpress(
        &mut self,
        button: Button,
        is_pressed: bool,
    ) -> Result<Option<InputEvent>, InputError> {
        if !is_pressed {
            self.state = HandlerState::Normal;
            Ok(Some(InputEvent { button, event_type: InputEventType::Release }))
        } else {
            Ok(None)
        }
    }

    fn handle_menu(
        &mut self,
        button: Button,
        is_pressed: bool,
    ) -> Result<Option<InputEvent>, InputError> {
        if !is_pressed {
            return Ok(None);
        }
        match button {
            Button::Up | Button::Down => {
                Ok(Some(InputEvent { button, event_type: InputEventType::Press }))
            }
            Button::Left | Button::Right => {
                self.state = HandlerState::Normal;
                Ok(Some(InputEvent { button, event_type: InputEventType::Release }))
            }
            _ => Ok(None),
        }
    }

    pub fn last_input_ms(&self) -> u32 {
        self.last_input_ms
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
