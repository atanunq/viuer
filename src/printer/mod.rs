use crate::config::Config;
use crate::utils::terminal_size;
use image::{DynamicImage, GenericImageView};

mod block;
mod kitty;

pub use block::BlockPrinter;
pub use kitty::has_kitty_support;
pub use kitty::KittyPrinter;
pub use kitty::KittySupport;

pub trait Printer {
    fn print(img: &DynamicImage, config: &Config) -> crate::ViuResult;
}

// Helper method that resizes a [image::DynamicImage]
// to make it fit in the terminal.
//
// The behaviour is different based on the provided width and height:
// - If both are None, the image will be resized to fit in the terminal. Aspect ratio is preserved.
// - If only one is provided and the other is None, it will fit the image in the provided boundary. Aspect ratio is preserved.
// - If both are provided, the image will be resized to match the new size. Aspect ratio is **not** preserved.
//
// Note that if the image is smaller than the available space, no transformations will be made.
fn resize(img: &DynamicImage, width: Option<u32>, height: Option<u32>) -> DynamicImage {
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
            let (term_w, term_h) = terminal_size();
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

/// Figure out the best dimensions for the printed image, based on user's input.
/// Returns the dimensions of how the image should be printed in terminal cells.
fn find_best_fit(img: &DynamicImage, width: Option<u32>, height: Option<u32>) -> (u16, u16) {
    let (img_width, img_height) = img.dimensions();

    // The height of a terminal cell is twice its width. Thus, when calling fit_dimensions,
    //  it has to know that the actual dimensions are (w, 2*h), so that it can correctly
    //  maintain the aspect ratio.
    //  The return height, however, represents terminal cells and has to be divided by two.
    //
    // For example, to fit image 160x80 in a terminal grid 80x24:
    // Aspect ratio is 160/80 = 2:1,
    // the terminal can be split into 80x46 equivalent squares, and
    // the image has to be scaped down. Preserving aspect ratio,
    // the maximum fit will be 80x40 squares, or 80x20 terminal cells.
    // Also, after 1 extra row is kept (see below), the returned dimensions will be 80x19.

    // Match user's width and height preferences
    match (width, height) {
        (None, None) => {
            let (term_w, term_h) = terminal_size();
            let (w, h) = fit_dimensions(img_width, img_height, term_w as u32, 2 * term_h as u32);

            //TODO: is -1 row always necessary? Maybe only when term_h*2 == h?
            // One less row because two reasons:
            // - the prompt after executing the command will take a line
            // - gifs flicker
            (w as u16, (h / 2 - 1) as u16)
        }
        // Either width or height is specified, will fit and preserve aspect ratio.
        (Some(w), None) => {
            let (w, h) = fit_dimensions(img_width, img_height, w, 2 * img_height);
            (w as u16, (h / 2) as u16)
        }
        (None, Some(h)) => {
            let (w, h) = fit_dimensions(img_width, img_height, img_width, 2 * h);
            (w as u16, (h / 2) as u16)
        }
        (Some(w), Some(h)) => {
            // Both width and height are specified, will resize to match exactly
            (w as u16, h as u16)
        }
    }
}

/// Given width & height of an image, scale the size so that it can fit within given bounds
/// while preserving aspect ratio. Will scale both up and down.
fn fit_dimensions(width: u32, height: u32, bound_width: u32, bound_height: u32) -> (u32, u32) {
    let ratio = width * bound_height;
    let nratio = bound_width * height;

    let use_width = nratio <= ratio;
    let intermediate = if use_width {
        height * bound_width / width
    } else {
        width * bound_height / height
    };

    if use_width {
        (bound_width, intermediate)
    } else {
        (intermediate, bound_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn best_fit_large_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(600, 500))
    }

    fn best_fit_small_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(40, 25))
    }

    #[test]
    fn find_best_fit_none() {
        let width = None;
        let height = None;

        let img = best_fit_large_test_image();
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 57);
        assert_eq!(h, 23);

        let img = best_fit_small_test_image();
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 76);
        assert_eq!(h, 23);

        let img = DynamicImage::ImageRgba8(image::RgbaImage::new(160, 80));
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 80);
        assert_eq!(h, 19);
    }

    #[test]
    fn find_best_fit_some_none() {
        let width = Some(100);
        let height = None;

        let img = best_fit_large_test_image();
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 100);
        assert_eq!(h, 41);

        let img = best_fit_small_test_image();
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 80);
        assert_eq!(h, 25);

        let width = Some(6);
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 6);
        assert_eq!(h, 1);
    }

    #[test]
    fn find_best_fit_none_some() {
        let width = None;
        let height = Some(90);

        let img = best_fit_large_test_image();
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 216);
        assert_eq!(h, 90);

        let height = Some(4);
        let img = best_fit_small_test_image();
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 12);
        assert_eq!(h, 4);
    }

    #[test]
    fn find_best_fit_some_some() {
        let width = Some(15);
        let height = Some(9);

        let img = best_fit_large_test_image();
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 15);
        assert_eq!(h, 9);

        let img = best_fit_small_test_image();
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 15);
        assert_eq!(h, 9);
    }

    #[test]
    fn test_fit() {
        // ratio 1:1
        assert_eq!((40, 40), fit_dimensions(100, 100, 40, 50));
        assert_eq!((30, 30), fit_dimensions(100, 100, 40, 30));
        // ratio 3:2
        assert_eq!((30, 20), fit_dimensions(240, 160, 30, 100));
        // ratio 5:7
        assert_eq!((100, 140), fit_dimensions(300, 420, 320, 140));
        // ratio 4:3
        assert_eq!((32, 24), fit_dimensions(4, 3, 80, 24));
    }

    fn resize_get_large_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(1000, 800))
    }

    fn resize_get_small_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(20, 10))
    }

    #[test]
    fn test_resize_none() {
        let width = None;
        let height = None;

        let img = resize_get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 57);
        assert_eq!(new_img.height(), 46);

        let img = resize_get_small_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 20);
        assert_eq!(new_img.height(), 10);
    }

    #[test]
    fn test_resize_some_none() {
        let width = Some(100);
        let height = None;

        let img = resize_get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 100);
        assert_eq!(new_img.height(), 80);

        let img = resize_get_small_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 20);
        assert_eq!(new_img.height(), 10);
    }

    #[test]
    fn test_resize_none_some() {
        let width = None;
        let mut height = Some(90);

        let img = resize_get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 225);
        assert_eq!(new_img.height(), 180);

        height = Some(4);
        let img = resize_get_small_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 16);
        assert_eq!(new_img.height(), 8);
    }

    #[test]
    fn test_resize_some_some() {
        let width = Some(15);
        let height = Some(9);

        let img = resize_get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 15);
        assert_eq!(new_img.height(), 18);

        let img = resize_get_small_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 15);
        assert_eq!(new_img.height(), 18);
    }
}
