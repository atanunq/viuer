use crate::config::Config;
use image::DynamicImage;

mod block;
pub use block::BlockPrinter;

pub trait Printer {
    fn print(img: &DynamicImage, config: &Config) -> crate::ViuResult;
}
