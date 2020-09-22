use crate::utils;

/// Configuration struct to customize printing behaviour.
pub struct Config {
    pub transparent: bool,
    pub truecolor: bool,
    pub resize: bool,
    //TODO: move the image to match passed x and y
    pub x: u32,
    pub y: u32,
    pub width: Option<u32>,
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
