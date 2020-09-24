use crate::utils;

/// Configuration struct to customize printing behaviour.
pub struct Config {
    /// Enable true transparency instead of checkerboard background. Defaults to false.
    pub transparent: bool,
    /// Use truecolor when the terminal supports it. Defaults to true.
    pub truecolor: bool,
    /// Resize the image before printing. Defaults to true.
    pub resize: bool,
    /// X offset. Defaults to 0.
    pub x: u16,
    /// Y offset. Defaults to 0.
    pub y: u16,
    /// Optional image width. Defaults to None.
    pub width: Option<u32>,
    /// Optional image height. Defaults to None.
    pub height: Option<u32>,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            transparent: false,
            truecolor: utils::truecolor_available(),
            resize: true,
            x: 0,
            y: 0,
            width: None,
            height: None,
        }
    }
}
