use crate::config::Config;
use image::DynamicImage;

mod block;
mod kitty;

pub use block::BlockPrinter;
pub use kitty::has_kitty_support;
pub use kitty::KittyPrinter;
pub use kitty::KittySupport;

pub trait Printer {
    fn print(img: &DynamicImage, config: &Config) -> crate::ViuResult;
}
