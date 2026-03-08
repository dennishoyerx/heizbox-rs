use crate::ScreenType;

<<<<<<< ours
/// Represents the active screen and any queued navigation.
pub struct ScreenState {
    pub current: ScreenType,
=======
pub struct ScreenState {
    pub current:            ScreenType,
>>>>>>> theirs
    pub pending_navigation: Option<ScreenType>,
}

impl ScreenState {
    pub fn new() -> Self {
<<<<<<< ours
        Self {
            current: ScreenType::Startup,
            pending_navigation: None,
        }
=======
        Self { current: ScreenType::Startup, pending_navigation: None }
>>>>>>> theirs
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
<<<<<<< ours
    fn default() -> Self {
        Self::new()
    }
=======
    fn default() -> Self { Self::new() }
>>>>>>> theirs
}
