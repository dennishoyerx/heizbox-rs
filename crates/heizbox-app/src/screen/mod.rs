pub mod fire;
pub mod menu;
pub mod nav;
pub mod startup;
pub mod state;

use async_trait::async_trait;
use heizbox_core::event::DomainEvent;
use crate::{Button, InputEvent, ScreenType};

// ── Framebuffer ───────────────────────────────────────────────────────────────

/// Raw 16-bit (RGB565) framebuffer for a 280×240 display.
pub struct FrameBuffer {
    pub data: heapless::Vec<u8, 134400>, // 280 * 240 * 2
    pub width: u16,
    pub height: u16,
}

impl FrameBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            data: heapless::Vec::new(),
            width,
            height,
        }
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

#[async_trait]
pub trait Screen: Send {
    async fn on_enter(&mut self);
    async fn on_exit(&mut self);
    async fn handle_input(&mut self, event: InputEvent) -> Result<Navigation, ScreenError>;
    async fn update(&mut self, event: DomainEvent) -> Result<(), ScreenError>;
    async fn render(&self) -> Result<FrameBuffer, ScreenError>;
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

/// Return a colour that represents the heating level for a given temperature.
pub fn temp_to_color(temp: u16) -> Rgb565 {
    match temp {
        0..=165 => Rgb565::from_rgb(30, 202, 211),  // cyan  — cold
        166..=180 => Rgb565::from_rgb(46, 204, 113), // green — flavour
        181..=195 => Rgb565::from_rgb(241, 196, 15), // yellow — balanced
        196..=215 => Rgb565::from_rgb(230, 126, 34), // orange — extraction
        _ => Rgb565::from_rgb(192, 57, 43),          // red — hot
    }
}
