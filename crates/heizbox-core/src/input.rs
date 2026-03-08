#[derive(Debug, Clone)]
pub enum InputEvent {
<<<<<<< ours
    ButtonPress { button: Button },
    ButtonLongPress { button: Button },
    ButtonRelease { button: Button },
    EncoderTurn { steps: i8 },
=======
    ButtonPress   { button: Button },
    ButtonLongPress { button: Button },
    ButtonRelease { button: Button },
    EncoderTurn   { steps: i8 },
>>>>>>> theirs
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
