#[derive(Debug, Clone)]
pub enum InputEvent {
    ButtonPress { button: Button },
    ButtonLongPress { button: Button },
    ButtonRelease { button: Button },
    EncoderTurn { steps: i8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Fire,
    Up,
    Down,
    Left,
    Right,
    Center,
}
