pub mod fire;
pub mod menu;
pub mod nav;
<<<<<<< ours
=======
pub mod ota;
pub mod screensaver;
>>>>>>> theirs
pub mod startup;
pub mod state;

use heizbox_core::event::DomainEvent;
<<<<<<< ours
use crate::{Button, InputEvent, ScreenType};
=======
use crate::{InputEvent, ScreenType};
>>>>>>> theirs

// ── Framebuffer ───────────────────────────────────────────────────────────────

/// Raw 16-bit (RGB565) framebuffer for a 240×280 display.
pub struct FrameBuffer {
<<<<<<< ours
    pub data: heapless::Vec<u8, 134400>, // 240 * 280 * 2
    pub width: u16,
=======
    pub data:   heapless::Vec<u8, 134400>, // 240 * 280 * 2
    pub width:  u16,
>>>>>>> theirs
    pub height: u16,
}

impl FrameBuffer {
<<<<<<< ours
    /// Create a new framebuffer with the given dimensions.
    /// The buffer is allocated to full size (width * height * 2) and initialized to zeros.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize) * 2;
        let mut data = heapless::Vec::new();
        // Pre-allocate full capacity (which is exactly 134400 for 240x280).
        // Resize will fill with zeros.
=======
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize) * 2;
        let mut data = heapless::Vec::new();
>>>>>>> theirs
        let _ = data.resize(size, 0);
        Self { data, width, height }
    }

<<<<<<< ours
    /// Create a new framebuffer with existing data (e.g., for testing).
=======
>>>>>>> theirs
    pub fn from_data(width: u16, height: u16, data: heapless::Vec<u8, 134400>) -> Self {
        Self { data, width, height }
    }
}

// ── Navigation result ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum Navigation {
    None,
    GoTo(ScreenType),
    Back,
    Exit,
}

// ── Screen errors ─────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ScreenError {
    #[error("Render failed")]
    RenderError,
    #[error("Event handling failed")]
    EventError,
}

// ── Screen trait ──────────────────────────────────────────────────────────────

pub trait Screen: Send {
    fn on_enter(&mut self);
    fn on_exit(&mut self);
    fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError>;
    fn update(&mut self, event: DomainEvent) -> Result<(), ScreenError>;
    fn render(&self) -> Result<FrameBuffer, ScreenError>;
}

// ── Color helper (RGB565) ─────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Rgb565(pub u16);

impl Rgb565 {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let r5 = (r >> 3) as u16;
        let g6 = (g >> 2) as u16;
        let b5 = (b >> 3) as u16;
        Self((r5 << 11) | (g6 << 5) | b5)
    }
}

<<<<<<< ours
/// Return a colour that represents the heating level for a given temperature.
pub fn temp_to_color(temp: u16) -> Rgb565 {
    match temp {
        0..=165 => Rgb565::from_rgb(30, 202, 211),  // cyan  — cold
        166..=180 => Rgb565::from_rgb(46, 204, 113), // green — flavour
        181..=195 => Rgb565::from_rgb(241, 196, 15), // yellow — balanced
        196..=215 => Rgb565::from_rgb(230, 126, 34), // orange — extraction
        _ => Rgb565::from_rgb(192, 57, 43),          // red — hot
=======
/// Map a temperature value to a display colour.
pub fn temp_to_color(temp: u16) -> Rgb565 {
    match temp {
        0..=165   => Rgb565::from_rgb(30, 202, 211),  // cyan   — cold
        166..=180 => Rgb565::from_rgb(46, 204, 113),  // green  — flavour
        181..=195 => Rgb565::from_rgb(241, 196, 15),  // yellow — balanced
        196..=215 => Rgb565::from_rgb(230, 126, 34),  // orange — extraction
        _         => Rgb565::from_rgb(192, 57, 43),   // red    — hot
>>>>>>> theirs
    }
}
