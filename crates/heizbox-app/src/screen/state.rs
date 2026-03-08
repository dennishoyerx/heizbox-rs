use crate::ScreenType;

pub struct ScreenState {
    pub current:            ScreenType,
    pub pending_navigation: Option<ScreenType>,
}

impl ScreenState {
    pub fn new() -> Self {
        Self { current: ScreenType::Startup, pending_navigation: None }
    }

    pub fn request_navigation(&mut self, target: ScreenType) {
        self.pending_navigation = Some(target);
    }

    pub fn apply_navigation(&mut self) -> Option<ScreenType> {
        let next = self.pending_navigation.take()?;
        self.current = next;
        Some(next)
    }
}

impl Default for ScreenState {
    fn default() -> Self { Self::new() }
}
