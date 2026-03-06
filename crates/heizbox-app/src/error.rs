use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Screen error: {0}")]
    Screen(String),
    #[error("Input error: {0}")]
    Input(String),
    #[error("Navigation error: {0}")]
    Navigation(String),
}
