use crate::utils;

pub struct Config {
    pub transparent: bool,
    pub truecolor: bool,
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
            x: 0,
            y: 0,
            width: None,
            height: None,
        }
    }
}
