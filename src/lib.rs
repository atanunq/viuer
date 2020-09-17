use crossterm::terminal;
use error::ViuResult;
use image::{DynamicImage, GenericImageView};

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
    let resized_img = resize(&img, config);
    print(&resized_img, config)
}

fn resize(img: &DynamicImage, config: &Config) -> DynamicImage {
    let new_img;
    let (width, height) = img.dimensions();
    let (mut print_width, mut print_height) = (width, height);

    if let Some(w) = config.width {
        print_width = w;
    }
    if let Some(h) = config.height {
        //since 2 pixels are printed per terminal cell, an image with twice the height can be fit
        print_height = 2 * h;
    }
    match (config.width, config.height) {
        (None, None) => {
            let size;
            match terminal::size() {
                Ok(s) => {
                    size = s;
                }
                Err(_) => {
                    //If getting terminal size fails, fall back to some default size
                    size = (100, 40);
                }
            }
            let (term_w, term_h) = size;
            let w = u32::from(term_w);
            //One less row because two reasons:
            // - the prompt after executing the command will take a line
            // - gifs flicker
            let h = u32::from(term_h - 1);
            if width > w {
                print_width = w;
            }
            if height > h {
                print_height = 2 * h;
            }
            new_img = img.thumbnail(print_width, print_height);
        }
        (Some(_), None) | (None, Some(_)) => {
            //Either width or height is specified, resizing and preserving aspect ratio
            new_img = img.thumbnail(print_width, print_height);
        }
        (Some(_w), Some(_h)) => {
            //Both width and height are specified, resizing without preserving aspect ratio
            new_img = img.thumbnail_exact(print_width, print_height);
        }
    };

    new_img
}

//TODO: resize tests :)
