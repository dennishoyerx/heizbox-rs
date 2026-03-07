use heizbox_app::screen::FrameBuffer;

/// Display output trait.
/// Implemented by concrete display drivers (e.g., SPI-based displays).
pub trait Display {
    /// Flush a framebuffer to the display hardware.
    /// This function should handle the actual pixel transfer (e.g., via SPI DMA).
    fn flush(&mut self, fb: &FrameBuffer);
}