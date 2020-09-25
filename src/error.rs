pub type ViuResult = std::result::Result<(), ViuError>;

/// Custom error type for `viu`ing operations.
#[derive(Debug)]
pub enum ViuError {
    /// Encountered an error while doing transformations with the [`image`] crate.
    Image(image::ImageError),
    /// Encountered an error while doing IO operations.
    IO(std::io::Error),
    /// Encountered an error while doing [`crossterm`] operations.
    Crossterm(crossterm::ErrorKind),
    /// Invalid configuration provided.
    InvalidConfiguration(String),
}

impl std::error::Error for ViuError {}

impl From<std::io::Error> for ViuError {
    fn from(err: std::io::Error) -> Self {
        ViuError::IO(err)
    }
}
impl From<image::ImageError> for ViuError {
    fn from(err: image::ImageError) -> Self {
        ViuError::Image(err)
    }
}

impl From<crossterm::ErrorKind> for ViuError {
    fn from(err: crossterm::ErrorKind) -> Self {
        ViuError::Crossterm(err)
    }
}

impl std::fmt::Display for ViuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViuError::Image(e) => write!(f, "Image error: {}", e),
            ViuError::IO(e) => write!(f, "IO error: {}", e),
            ViuError::Crossterm(e) => write!(f, "Crossterm error: {}", e),
            ViuError::InvalidConfiguration(s) => write!(f, "Invalid Configuration: {}", s),
        }
    }
}
