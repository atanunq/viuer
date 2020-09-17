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
    let resized_img = resize(&img, &config);
    //TODO: logic to choose printer (Sixel, etc.)
    printer::BlockPrinter::print(&resized_img, config)
}

pub fn print_from_file(filename: &str, config: &Config) -> ViuResult {
    let img = image::io::Reader::open(filename)?
        .with_guessed_format()?
        .decode()?;
    let resized_img = resize(&img, config);
    print(&resized_img, config)
}

pub fn resize(img: &DynamicImage, config: &Config) -> DynamicImage {
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
            let (term_w, term_h) = utils::terminal_size();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use image::DynamicImage;

    fn get_large_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(1000, 800))
    }

    fn get_small_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(20, 10))
    }

    #[test]
    fn test_resize_none() {
        let config = Config {
            width: None,
            height: None,
            ..Default::default()
        };

        let img = get_large_test_image();
        let new_img = resize(&img, &config);
        assert_eq!(new_img.width(), 97);
        assert_eq!(new_img.height(), 78);

        let img = get_small_test_image();
        let new_img = resize(&img, &config);
        assert_eq!(new_img.width(), 20);
        assert_eq!(new_img.height(), 10);
    }

    #[test]
    fn test_resize_some_none() {
        let config = Config {
            width: Some(100),
            height: None,
            ..Default::default()
        };

        let img = get_large_test_image();
        let new_img = resize(&img, &config);
        assert_eq!(new_img.width(), 100);
        assert_eq!(new_img.height(), 80);

        let config = Config {
            width: Some(100),
            height: None,
            ..Default::default()
        };
        let img = get_small_test_image();
        let new_img = resize(&img, &config);
        assert_eq!(new_img.width(), 20);
        assert_eq!(new_img.height(), 10);
    }

    #[test]
    fn test_resize_none_some() {
        let config = Config {
            width: None,
            height: Some(90),
            ..Default::default()
        };

        let img = get_large_test_image();
        let new_img = resize(&img, &config);
        assert_eq!(new_img.width(), 225);
        assert_eq!(new_img.height(), 180);

        let config = Config {
            width: None,
            height: Some(4),
            ..Default::default()
        };
        let img = get_small_test_image();
        let new_img = resize(&img, &config);
        assert_eq!(new_img.width(), 16);
        assert_eq!(new_img.height(), 8);
    }

    #[test]
    fn test_resize_some_some() {
        let config = Config {
            width: Some(15),
            height: Some(9),
            ..Default::default()
        };

        let img = get_large_test_image();
        let new_img = resize(&img, &config);
        assert_eq!(new_img.width(), 15);
        assert_eq!(new_img.height(), 18);

        let img = get_small_test_image();
        let new_img = resize(&img, &config);
        assert_eq!(new_img.width(), 15);
        assert_eq!(new_img.height(), 18);
    }
}
