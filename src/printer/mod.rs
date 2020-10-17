use crate::config::Config;
use image::DynamicImage;

mod block;
mod kitty;

pub use block::BlockPrinter;
pub use kitty::KittyPrinter;
pub use kitty::KittySupport;

pub trait Printer {
    fn print(img: &DynamicImage, config: &Config) -> crate::ViuResult;
}

/// Returns the terminal's support for the Kitty graphics protocol.
pub fn has_kitty_support() -> KittySupport {
    *kitty::KITTY_SUPPORT
}
