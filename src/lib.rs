use error::ViuResult;
use image::DynamicImage;

mod config;
mod error;
mod printer;
mod utils;

pub use config::Config;
use printer::Printer;

/// Default printing method. Uses upper and lower half blocks to fill
/// terminal cells.
pub fn print(img: &DynamicImage, config: &Config) -> ViuResult {
    //TODO: width and height logic
    //TODO: logic to choose printer (Sixel, etc.)
    printer::BlockPrinter::print(img, config)
}

pub fn print_from_file(filename: &str, config: &Config) -> ViuResult {
    let img = image::io::Reader::open(filename)?
        .with_guessed_format()?
        .decode()?;
    print(&img, config)
}
