use image::DynamicImage;

mod config;
mod printer;
mod utils;

pub use config::Config;
use printer::Printer;

type ViuResult = std::result::Result<(), ViuError>;

#[derive(Debug)]
pub enum ViuError {
    Print,
}

impl std::fmt::Display for ViuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ViuError::Print => "Writing to stdout was unsuccessful.",
        };
        write!(f, "{}", msg)
    }
}

/// Default printing method. Uses upper and lower half blocks to fill
/// terminal cells.
pub fn print(img: &DynamicImage, config: &Config) -> ViuResult {
    //TODO: width and height logic
    printer::BlockPrinter::print(img, config)
}
