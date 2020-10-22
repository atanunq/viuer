use crate::config::Config;
use crate::error::ViuResult;
use crate::utils::terminal_size;
use image::{DynamicImage, GenericImageView};

mod block;
mod kitty;

pub use block::BlockPrinter;
pub use kitty::has_kitty_support;
pub use kitty::KittyPrinter;
pub use kitty::KittySupport;

pub trait Printer {
    // Print the given image in the terminal while respecting the options in the config struct.
    // Return the dimensions of the printed image in **terminal cells**.
    fn print(img: &DynamicImage, config: &Config) -> ViuResult<(u32, u32)>;
}

/// Resize a [image::DynamicImage] so that it fits within optional width and height bounds.
/// If none are provided, terminal size is used instead.
pub fn resize(img: &DynamicImage, width: Option<u32>, height: Option<u32>) -> DynamicImage {
    let (w, h) = find_best_fit(img, width, height);

    // find_best_fit returns values in terminal cells. Hence we multiply by two
    // because a 5x10 image can fit in 5x5 cells.
    img.resize_exact(w, 2 * h, image::imageops::FilterType::Triangle)
}

/// Find the best dimensions for the printed image, based on user's input.
/// Returns the dimensions of how the image should be printed in **terminal cells**.
///
/// The behaviour is different based on the provided width and height:
/// - If both are None, the image will be resized to fit in the terminal. Aspect ratio is preserved.
/// - If only one is provided and the other is None, it will fit the image in the provided boundary. Aspect ratio is preserved.
/// - If both are provided, the image will be resized to match the new size. Aspect ratio is **not** preserved.
///
/// Example:
/// Use None for both dimensions to use terminal size (80x24) instead.
/// The image ratio is 2:1, the terminal can be split into 80x46 squares.
/// The best fit would be to use the whole width (80) and 40 vertical squares,
/// which is equivalent to 20 terminal cells.
///
/// let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(160, 80));
/// let (w, h) = find_best_fit(&img, None, None);
/// assert_eq!(w, 80);
/// assert_eq!(h, 20);
fn find_best_fit(img: &DynamicImage, width: Option<u32>, height: Option<u32>) -> (u32, u32) {
    let (img_width, img_height) = img.dimensions();

    // Match user's width and height preferences
    match (width, height) {
        (None, None) => {
            let (term_w, term_h) = terminal_size();
            let (w, h) = fit_dimensions(img_width, img_height, term_w as u32, term_h as u32);

            // One less row because two reasons:
            // - the prompt after executing the command will take a line
            // - gifs flicker
            let h = if h == term_h as u32 { h - 1 } else { h };
            (w, h)
        }
        // Either width or height is specified, will fit and preserve aspect ratio.
        (Some(w), None) => fit_dimensions(img_width, img_height, w, img_height),
        (None, Some(h)) => fit_dimensions(img_width, img_height, img_width, h),

        // Both width and height are specified, will resize to match exactly
        (Some(w), Some(h)) => (w, h),
    }
}

/// Given width & height of an image, scale the size so that it can fit within given bounds
/// while preserving aspect ratio. Will only scale down - if dimensions are smaller than the
/// bounds, they will be returned unmodified.
///
/// Note: input bounds are meant to hold dimensions of a terminal, where the height of a cell is
/// twice it's width. It is best illustrated in an example:
///
/// Trying to fit a 100x100 image in 40x15 terminal cells. The best fit, while having an aspect
/// ratio of 1:1, would be to use all of the available height, 15, which is
/// equivalent in size to 30 vertical cells. Hence, the returned dimensions will be 30x15.
///
/// assert_eq!((30, 15), viuer::fit_dimensions(100, 100, 40, 15));
fn fit_dimensions(width: u32, height: u32, bound_width: u32, bound_height: u32) -> (u32, u32) {
    let bound_height = 2 * bound_height;

    if width <= bound_width && height <= bound_height {
        return (width, std::cmp::max(1, height / 2));
    }

    let ratio = width * bound_height;
    let nratio = bound_width * height;

    let use_width = nratio <= ratio;
    let intermediate = if use_width {
        height * bound_width / width
    } else {
        width * bound_height / height
    };

    if use_width {
        (bound_width, std::cmp::max(1, intermediate / 2))
    } else {
        (intermediate, std::cmp::max(1, bound_height / 2))
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

    fn resize_get_large_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(1000, 800))
    }

    fn resize_get_small_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(20, 10))
    }

    // Resize tests

    #[test]
    fn test_resize_none() {
        let width = None;
        let height = None;

        let img = resize_get_large_test_image();
        let new_img = resize(&img, width, height);
        assert_eq!(new_img.width(), 60);
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

    // Best fit tests

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
        assert_eq!(w, 40);
        assert_eq!(h, 12);

        let img = DynamicImage::ImageRgba8(image::RgbaImage::new(160, 80));
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 80);
        assert_eq!(h, 20);
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
        assert_eq!(w, 40);
        assert_eq!(h, 12);

        let width = Some(6);
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 6);
        assert_eq!(h, 1);

        let width = Some(3);
        let (w, h) = find_best_fit(&img, width, height);
        assert_eq!(w, 3);
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
    fn test_fit_dimensions() {
        // ratio 1:1
        assert_eq!((40, 20), fit_dimensions(100, 100, 40, 50));
        assert_eq!((20, 10), fit_dimensions(100, 100, 40, 10));
        // ratio 3:2
        assert_eq!((30, 10), fit_dimensions(240, 160, 30, 100));
        // ratio 5:7
        assert_eq!((200, 140), fit_dimensions(300, 420, 320, 140));
    }

    #[test]
    fn test_fit_smaller_than_bounds() {
        assert_eq!((4, 1), fit_dimensions(4, 3, 80, 24));
        assert_eq!((4, 1), fit_dimensions(4, 1, 80, 24));
    }

    #[test]
    fn test_fit_equal_to_bounds() {
        assert_eq!((80, 12), fit_dimensions(80, 24, 80, 24));
    }
}
