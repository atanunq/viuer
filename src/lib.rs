#![deny(missing_docs)]
// Temporary until https://github.com/rust-lang/rust-clippy/issues/15151 is rolled out
#![allow(clippy::uninlined_format_args)]

//! Library to display images in the terminal.
//!
//! This library contains functionality extracted from the [`viu`](https://github.com/atanunq/viu) crate.
//! It aims to provide an easy to use interface to print images in the terminal. Uses some abstractions
//! provided by the [`image`] crate. [Kitty](https://sw.kovidgoyal.net/kitty/graphics-protocol.html)
//! and [iTerm](https://iterm2.com/documentation-images.html) graphic protocols are supported and used by default,
//! if detected. If not, `viuer` will fallback to using regular half blocks instead (▄ and ▀).
//!
//! ## Basic Usage
//! The default features of this crate can only work with [image::DynamicImage]. The below example
//! creates a 60x60 gradient and prints it. More options are available through the [Config] struct.
//! ```
//! use image::{DynamicImage, Pixel, Rgba, RgbaImage};
//!
//! let conf = viuer::Config {
//!     absolute_offset: false,
//!     ..Default::default()
//! };
//!
//! let mut img = DynamicImage::ImageRgba8(RgbaImage::new(60, 60));
//! let start = Rgba::from_slice(&[0, 196, 0, 255]);
//! let end = Rgba::from_slice(&[255, 255, 255, 255]);
//! image::imageops::horizontal_gradient(&mut img, start, end);
//!
//! viuer::print(&img, &conf).unwrap();
//! ```
//!
//! ## Decoding files
//! To work directly with files, the non-default `print-file` feature must be enabled.
//!
//! The example below shows how to print the image `img.jpg` in 40x30 terminal cells, with vertical
//! offset of 4 and horizontal of 10, starting from the top left corner.
//! ```no_run
//! let conf = viuer::Config {
//!     width: Some(40),
//!     height: Some(30),
//!     x: 10,
//!     y: 4,
//!     ..Default::default()
//! };
//!
//! #[cfg(feature="print-file")]
//! viuer::print_from_file("img.jpg", &conf).expect("Image printing failed.");
//! ```

#[cfg(feature = "print-file")]
use std::path::Path;

use crossterm::{
    cursor::{RestorePosition, SavePosition},
    execute,
};
use image::DynamicImage;
use printer::{Printer, PrinterType};

mod config;
mod error;
mod printer;
mod utils;

pub use config::Config;
pub use error::{ViuError, ViuResult};
pub use printer::{get_kitty_support, is_iterm_supported, resize, KittySupport};
pub use utils::terminal_size;

#[cfg(feature = "sixel")]
pub use printer::is_sixel_supported;

/// Default printing method. Uses either iTerm or Kitty graphics protocol, if supported,
/// and half blocks otherwise.
///
/// Check the [Config] struct for all customization options.
/// ## Example
/// The snippet below reads all of stdin, decodes it with the [`image`] crate
/// and prints it to the terminal. The image will also be resized to fit in the terminal.
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
/// print(&img, &Config::default()).expect("Image printing failed.");
/// ```
pub fn print(img: &DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
    let mut stdout = std::io::stdout();
    if config.restore_cursor {
        execute!(&mut stdout, SavePosition)?;
    }

    let (w, h) = choose_printer(config).print(&mut stdout, img, config)?;

    if config.restore_cursor {
        execute!(&mut stdout, RestorePosition)?;
    };

    Ok((w, h))
}

/// Helper method that reads a file, tries to decode and print it. The feature is available only
/// with the `print-file` feature.
///
/// ## Example
/// ```no_run
/// use viuer::{Config, print_from_file};
/// let conf = Config {
///     width: Some(30),
///     transparent: true,
///     ..Default::default()
/// };
/// // Image will be scaled down to width 30. Aspect ratio will be preserved.
/// // Also, the terminal's background color will be used instead of checkerboard pattern.
/// print_from_file("img.jpg", &conf).expect("Image printing failed.");
/// ```
#[cfg(feature = "print-file")]
pub fn print_from_file<P: AsRef<Path>>(filename: P, config: &Config) -> ViuResult<(u32, u32)> {
    let mut stdout = std::io::stdout();
    if config.restore_cursor {
        execute!(&mut stdout, SavePosition)?;
    }

    let (w, h) = choose_printer(config).print_from_file(&mut stdout, filename, config)?;

    if config.restore_cursor {
        execute!(&mut stdout, RestorePosition)?;
    };

    Ok((w, h))
}

// Choose the appropriate printer to use based on user config and availability
fn choose_printer(config: &Config) -> PrinterType {
    #[cfg(feature = "sixel")]
    if config.use_sixel && is_sixel_supported() {
        return PrinterType::Sixel;
    }

    if config.use_iterm && is_iterm_supported() {
        PrinterType::iTerm
    } else if config.use_kitty && get_kitty_support() != KittySupport::None {
        PrinterType::Kitty
    } else {
        PrinterType::Block
    }
}
