use crate::error::{ViuError, ViuResult};
use crate::printer::Printer;
use crate::utils::{fit_dimensions, terminal_size};
use console::{Key, Term};
use crossterm::cursor::{MoveRight, MoveTo, MoveToPreviousLine};
use crossterm::execute;
use image::{DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use std::io::Write;

pub struct KittyPrinter {}

lazy_static! {
    pub static ref KITTY_SUPPORT: KittySupport = check_kitty_support();
}

/// Returns the terminal's support for the Kitty graphics protocol.
pub fn has_kitty_support() -> KittySupport {
    *KITTY_SUPPORT
}

impl Printer for KittyPrinter {
    fn print(img: &image::DynamicImage, config: &crate::Config) -> ViuResult {
        match *KITTY_SUPPORT {
            KittySupport::None => {
                // give up, print blocks
                Err(ViuError::KittyNotSupported)
            }
            KittySupport::Local => {
                // print from file
                print_local(img, config)
            }
            KittySupport::Remote => {
                // print through escape codes
                todo!()
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
/// The extend to which the Kitty graphics protocol can be used.
pub enum KittySupport {
    /// The Kitty graphics protocol is not supported.
    None,
    /// Kitty is running locally, data can be shared through a file.
    Local,
    /// Kitty is not running locally, data has to be sent through escape codes.
    Remote,
}

// Check if Kitty protocol can be used
fn check_kitty_support() -> KittySupport {
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("kitty") {
            if has_local_support().is_ok() {
                return KittySupport::Local;
            } else {
                return KittySupport::Remote;
            }
        }
    }
    KittySupport::None
}

// Query the terminal whether it can display an image from a file
fn has_local_support() -> ViuResult {
    // create a temp file that will hold a 1x1 image
    let x = image::RgbaImage::new(1, 1);
    let raw_img = x.as_raw();
    let path = store_in_tmp_file(raw_img)?;

    // send the query
    print!(
        // t=t tells Kitty it's reading from a temp file and will delete if afterwards
        "\x1b_Gi=31,s=1,v=1,a=q,t=t;{}\x1b\\",
        base64::encode(path.to_str().ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not convert path to &str"
        ))?)
    );
    std::io::stdout().flush()?;

    // collect Kitty's response after the query
    let term = Term::stdout();
    let mut response = Vec::new();

    while let Ok(key) = term.read_key() {
        // the response will end with Esc('x1b'), followed by Backslash('\')
        let should_break = key == Key::UnknownEscSeq(vec!['\\']);
        response.push(key);
        if should_break {
            break;
        }
    }

    // Kitty response should end with these 3 Keys if it was successful
    let expected = [
        Key::Char('O'),
        Key::Char('K'),
        Key::UnknownEscSeq(vec!['\\']),
    ];

    // check whether the last 3 Keys match the expected
    for x in response.windows(3).rev().take(1) {
        if x == expected {
            return Ok(());
        }
    }

    Err(ViuError::KittyResponse(response))
}

// Print with kitty graphics protocol through a temp file
// TODO: try with kitty's supported compression
fn print_local(img: &image::DynamicImage, config: &crate::Config) -> ViuResult {
    let rgba = img.to_rgba();
    let raw_img = rgba.as_raw();
    let path = store_in_tmp_file(raw_img)?;

    let mut stdout = std::io::stdout();
    // adjust offset
    if config.absolute_offset {
        if config.y >= 0 {
            // If absolute_offset, move to (x,y).
            execute!(&mut stdout, MoveTo(config.x, config.y as u16))?
        } else {
            //Negative values do not make sense.
            return Err(ViuError::InvalidConfiguration(
                "absolute_offset is true but y offset is negative".to_owned(),
            ));
        }
    } else if config.y < 0 {
        // MoveUp if negative
        execute!(&mut stdout, MoveToPreviousLine(-config.y as u16))?;
        execute!(&mut stdout, MoveRight(config.x))?;
    } else {
        // Move down y lines
        for _ in 0..config.y {
            // writeln! is used instead of MoveDown to force scrolldown
            // observed when config.y > 0 and cursor is on the last terminal line
            writeln!(&mut stdout)?
        }
        execute!(&mut stdout, MoveRight(config.x))?;
    }

    // get the desired width and height
    let (w, h) = resize(&img, config.width, config.height);

    print!(
        "\x1b_Gf=32,s={},v={},c={},r={},a=T,t=t;{}\x1b\\",
        img.width(),
        img.height(),
        w,
        h,
        base64::encode(path.to_str().unwrap())
    );
    println!();
    stdout.flush().unwrap();

    Ok(())
}
//TODO default_features false of console

// Figure out the best dimensions for the printed image, based on user's input.
// Returns the dimensions of how the image should be printed in terminal cells.
fn resize(img: &DynamicImage, width: Option<u32>, height: Option<u32>) -> (u16, u16) {
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

// Create a file in temporary dir and write the byte slice to it.
// Since the file is persisted, the user is responsible for deleting it afterwards.
fn store_in_tmp_file(raw_img: &[u8]) -> std::result::Result<std::path::PathBuf, ViuError> {
    let (mut tmpfile, path) = tempfile::Builder::new()
        .prefix(".tmp.viuer.")
        .rand_bytes(1)
        .tempfile()?
        .keep()?;

    tmpfile.write_all(raw_img).unwrap();
    tmpfile.flush().unwrap();
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_large_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(600, 500))
    }

    fn get_small_test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::new(40, 25))
    }

    #[test]
    fn test_resize_none() {
        let width = None;
        let height = None;

        let img = get_large_test_image();
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 57);
        assert_eq!(h, 23);

        let img = get_small_test_image();
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 76);
        assert_eq!(h, 23);

        let img = DynamicImage::ImageRgba8(image::RgbaImage::new(160, 80));
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 80);
        assert_eq!(h, 19);
    }

    #[test]
    fn test_resize_some_none() {
        let width = Some(100);
        let height = None;

        let img = get_large_test_image();
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 100);
        assert_eq!(h, 41);

        let img = get_small_test_image();
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 80);
        assert_eq!(h, 25);

        let width = Some(6);
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 6);
        assert_eq!(h, 1);
    }

    #[test]
    fn test_resize_none_some() {
        let width = None;
        let height = Some(90);

        let img = get_large_test_image();
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 216);
        assert_eq!(h, 90);

        let height = Some(4);
        let img = get_small_test_image();
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 12);
        assert_eq!(h, 4);
    }

    #[test]
    fn test_resize_some_some() {
        let width = Some(15);
        let height = Some(9);

        let img = get_large_test_image();
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 15);
        assert_eq!(h, 9);

        let img = get_small_test_image();
        let (w, h) = resize(&img, width, height);
        assert_eq!(w, 15);
        assert_eq!(h, 9);
    }
}
