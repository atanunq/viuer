#![deny(missing_docs)]
//! Small library to display images in the terminal.
//!
//! This library contains functionality extracted from the [`viu` crate](https://github.com/atanunq/viu).
//! It aims to provide an easy to use interface to print images in the terminal.
//!
//! ## Basic Usage
//! The example below shows how to print the image `img.jpg` in 40x60 terminal cells. Since
//! `viuer` uses half blocks by default (▄ and ▀), it will be able to fit a 40x120 image in 40x60 cells.
//! Options are available through the [Config](Config) struct.
//! ```
//! use viuer::{Config, print_from_file};
//! let conf = Config {
//!     width: Some(40),
//!     height: Some(60),
//!     ..Default::default()
//! };
//! print_from_file("img.jpg", &conf);
//! ```
//!

pub use error::ViuError;
use error::ViuResult;
use image::{DynamicImage, GenericImageView};

mod config;
mod error;
mod printer;
mod utils;

pub use config::Config;
use printer::Printer;

/// Default printing method. Uses upper and lower half blocks to fill terminal cells.
///
/// ## Example
/// The snippet below reads all of stdin, decodes it with the [image crate](https://docs.rs/image)
/// and prints it to the terminal.
///
/// ```no_run
/// use std::io::{stdin, Read};
/// use viuer::{Config, print};
///
/// let stdin = stdin();
/// let mut handle = stdin.lock();
///
/// let mut buf: Vec<u8> = Vec::new();
/// let _ = handle
///     .read_to_end(&mut buf)
///     .expect("Could not read until EOF.");
///
/// let img = image::load_from_memory(&buf).expect("Data from stdin could not be decoded.");
/// print(&img, &Config::default());
/// ```
pub fn print(img: &DynamicImage, config: &Config) -> ViuResult {
    // TODO: Could be extended to choose a different printer based
    // on availability

    if config.resize {
        let resized_img = resize(&img, config.width, config.height);
        printer::BlockPrinter::print(&resized_img, config)
    } else {
        printer::BlockPrinter::print(img, config)
    }
}

///Helper method that reads a file, tries to decode it and prints it.
pub fn print_from_file(filename: &str, config: &Config) -> ViuResult {
    let img = image::io::Reader::open(filename)?
        .with_guessed_format()?
        .decode()?;
    print(&img, config)
}

/// Helper method that resizes a [DynamicImage](https://docs.rs/image/*/image/enum.DynamicImage.html)
/// to make it fit in the terminal.
///
/// The behaviour is different based on the provided width and height:
/// - If both are None, the image will be resized to fit in the terminal. Aspect ratio is preserved.
/// - If only one is provided and the other is None, it will fit the image in the provided boundary. Aspect ratio is preserved.
/// - If both are provided, the image will be resized to match the new size. Aspect ratio is **not** preserved.
pub fn resize(img: &DynamicImage, width: Option<u32>, height: Option<u32>) -> DynamicImage {
    let (mut print_width, mut print_height) = img.dimensions();

    if let Some(w) = width {
        print_width = w;
    }
    if let Some(h) = height {
        //since 2 pixels are printed per terminal cell, an image with twice the height can be fit
        print_height = 2 * h;
    }
    match (width, height) {
        (None, None) => {
            let (term_w, term_h) = utils::terminal_size();
            let w = u32::from(term_w);
            // One less row because two reasons:
            // - the prompt after executing the command will take a line
            // - gifs flicker
            let h = u32::from(term_h - 1);
            if print_width > w {
                print_width = w;
            }
            if print_height > h {
                print_height = 2 * h;
            }
            img.thumbnail(print_width, print_height)
        }
        (Some(_), None) | (None, Some(_)) => {
            // Either width or height is specified, resizing and preserving aspect ratio
            img.thumbnail(print_width, print_height)
        }
        (Some(_), Some(_)) => {
            // Both width and height are specified, resizing without preserving aspect ratio
            img.thumbnail_exact(print_width, print_height)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    fn get_large_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(1000, 800))
    }

    fn get_small_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(20, 10))
    }

    #[test]
    fn test_resize_none() {
        let width = None;
        let height = None;

        let img = get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 97);
        assert_eq!(new_img.height(), 78);

        let img = get_small_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 20);
        assert_eq!(new_img.height(), 10);
    }

    #[test]
    fn test_resize_some_none() {
        let width = Some(100);
        let height = None;

        let img = get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 100);
        assert_eq!(new_img.height(), 80);

        let img = get_small_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 20);
        assert_eq!(new_img.height(), 10);
    }

    #[test]
    fn test_resize_none_some() {
        let width = None;
        let mut height = Some(90);

        let img = get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 225);
        assert_eq!(new_img.height(), 180);

        height = Some(4);
        let img = get_small_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 16);
        assert_eq!(new_img.height(), 8);
    }

    #[test]
    fn test_resize_some_some() {
        let width = Some(15);
        let height = Some(9);

        let img = get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 15);
        assert_eq!(new_img.height(), 18);

        let img = get_small_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 15);
        assert_eq!(new_img.height(), 18);
    }
}
