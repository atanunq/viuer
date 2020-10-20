#![deny(missing_docs)]

//! Small library to display images in the terminal.
//!
//! This library contains functionality extracted from the [`viu`](https://github.com/atanunq/viu) crate.
//! It aims to provide an easy to use interface to print images in the terminal. Uses some abstractions
//! provided by the [`image`] crate.
//!
//! ## Basic Usage
//! The example below shows how to print the image `img.jpg` in 40x30 terminal cells, starting at the
//! top left corner. Since `viuer` uses half blocks by default (▄ and ▀), it will be able to fit a
//! 40x60 image in 40x30 cells. Options are available through the [Config] struct.
//! ```no_run
//! use viuer::{Config, print_from_file};
//! let conf = Config {
//!     width: Some(40),
//!     height: Some(30),
//!     ..Default::default()
//! };
//! // will resize the image to fit in 40x60 terminal cells and print it
//! print_from_file("img.jpg", &conf).expect("Image printing failed.");
//! ```

use crossterm::cursor::{position, MoveToNextLine};
use crossterm::execute;
use crossterm::tty::IsTty;
use error::ViuResult;
use image::DynamicImage;
use printer::Printer;
use std::io::Write;

mod config;
mod error;
mod printer;
mod utils;

pub use config::Config;
pub use error::ViuError;
pub use printer::{has_kitty_support, KittySupport};
pub use utils::terminal_size;

/// Default printing method. Uses upper and lower half blocks to fill terminal cells.
///
/// ## Example
/// The snippet below reads all of stdin, decodes it with the [`image`] crate
/// and prints it to the terminal. The image will also be resized to fit in the terminal.
/// Check the [Config] struct if you would like to modify this behaviour.
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
pub fn print(img: &DynamicImage, config: &Config) -> ViuResult {
    let mut stdout = std::io::stdout();
    let is_tty = stdout.is_tty();
    // Only make note of cursor position in tty. Otherwise, it disturbes output in tools like `head`, for example.
    let cursor_pos = if is_tty { position().ok() } else { None };

    if config.use_kitty && has_kitty_support() != KittySupport::None {
        printer::KittyPrinter::print(img, config)?;
    } else {
        printer::BlockPrinter::print(img, config)?;
    }

    // if the cursor has ended above where it started, bring it back down to its lowest position
    if is_tty {
        if let Some((_, pos_y)) = cursor_pos {
            let (_, new_pos_y) = position()?;
            if pos_y > new_pos_y {
                execute!(&mut stdout, MoveToNextLine(pos_y - new_pos_y))?;
            };
        }
    };

    Ok(())
}

/// Helper method that reads a file, tries to decode it and prints it.
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
pub fn print_from_file(filename: &str, config: &Config) -> ViuResult {
    let img = image::io::Reader::open(filename)?
        .with_guessed_format()?
        .decode()?;
    print(&img, config)
}
