pub type ViuResult = std::result::Result<(), ViuError>;

/// Custom error type for `viu`ing operations.
#[derive(Debug)]
pub enum ViuError {
    /// Error while doing transformations with the [`image`] crate.
    Image(image::ImageError),
    /// Error while doing IO operations.
    IO(std::io::Error),
    /// Error while doing [`crossterm`] operations.
    Crossterm(crossterm::ErrorKind),
    /// Invalid configuration provided.
    InvalidConfiguration(String),
    /// Error while creating temp files
    Tempfile(tempfile::PersistError),
    /// Errenous response received from Kitty
    Kitty(Vec<console::Key>),
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

impl From<tempfile::PersistError> for ViuError {
    fn from(err: tempfile::PersistError) -> Self {
        ViuError::Tempfile(err)
    }
}

impl std::fmt::Display for ViuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViuError::Image(e) => write!(f, "Image error: {}", e),
            ViuError::IO(e) => write!(f, "IO error: {}", e),
            ViuError::Crossterm(e) => write!(f, "Crossterm error: {}", e),
            ViuError::InvalidConfiguration(s) => write!(f, "Invalid Configuration: {}", s),
            ViuError::Tempfile(e) => write!(f, "Tempfile error: {}", e),
            ViuError::Kitty(keys) => write!(f, "Kitty response: {:?}", keys),
        }
    }
}
