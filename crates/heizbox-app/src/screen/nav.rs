use crate::ScreenType;

#[derive(Debug)]
pub enum NavError {
    InvalidTransition { from: ScreenType, to: ScreenType },
    NoHistory,
}

impl core::fmt::Display for NavError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => {
                write!(f, "Cannot navigate from {from:?} to {to:?}")
            }
            Self::NoHistory => write!(f, "Navigation history is empty"),
        }
    }
}

/// Validates screen transitions.  Invalid transitions return `NavError` rather
/// than being silently ignored.
pub struct NavigationFsm {
    current: ScreenType,
    history: heapless::Vec<ScreenType, 8>,
}

impl NavigationFsm {
    pub fn new() -> Self {
        Self {
            current: ScreenType::Startup,
            history: heapless::Vec::new(),
        }
    }

    pub fn navigate_to(&mut self, target: ScreenType) -> Result<(), NavError> {
        self.validate_transition(self.current, target)?;
        let _ = self.history.push(self.current);
        self.current = target;
        Ok(())
    }

    pub fn navigate_back(&mut self) -> Result<(), NavError> {
        let prev = self.history.pop().ok_or(NavError::NoHistory)?;
        self.current = prev;
        Ok(())
    }

    pub fn current(&self) -> ScreenType {
        self.current
    }

    fn validate_transition(&self, from: ScreenType, to: ScreenType) -> Result<(), NavError> {
        let allowed: &[ScreenType] = match from {
            ScreenType::Startup     => &[ScreenType::Fire],
            ScreenType::Fire        => &[ScreenType::Fire, ScreenType::Menu, ScreenType::Screensaver],
            ScreenType::Menu        => &[ScreenType::Fire],
            ScreenType::Screensaver => &[ScreenType::Fire],
            ScreenType::OtaUpdate   => &[ScreenType::Fire],
        };
        if allowed.contains(&to) {
            Ok(())
        } else {
            Err(NavError::InvalidTransition { from, to })
        }
    }
}

impl Default for NavigationFsm {
    fn default() -> Self { Self::new() }
}
