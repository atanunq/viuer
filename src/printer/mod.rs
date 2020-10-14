use crate::config::Config;
use image::DynamicImage;

mod block;
mod kitty;

pub use block::BlockPrinter;
pub use kitty::KittyPrinter;

pub trait Printer {
    fn print(img: &DynamicImage, config: &Config) -> crate::ViuResult;
}
