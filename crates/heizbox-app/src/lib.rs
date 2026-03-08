pub mod device;
pub mod error;
pub mod event_bus;
pub mod input;
pub mod screen;

// ── Shared UI types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenType {
    Startup,
    Fire,
    Menu,
    Screensaver,
    OtaUpdate,
}

#[derive(Debug, Clone)]
pub struct InputEvent {
    pub button:     Button,
    pub event_type: InputEventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEventType {
    Press,
    LongPress,
    Release,
    Hold,
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
