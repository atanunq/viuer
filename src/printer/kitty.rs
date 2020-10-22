use crate::error::{ViuError, ViuResult};
use crate::printer::Printer;
use crate::Config;
//TODO default_features=false for console
use console::{Key, Term};
use crossterm::cursor::{MoveRight, MoveTo, MoveToPreviousLine};
use crossterm::execute;
use image::GenericImageView;
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
    fn print(img: &image::DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
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
fn print_local(img: &image::DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
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
    let (w, h) = super::find_best_fit(&img, config.width, config.height);

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

    Ok((w, h))
}

// Create a file in temporary dir and write the byte slice to it.
// Since the file is persisted, the user is responsible for deleting it afterwards.
fn store_in_tmp_file(buf: &[u8]) -> std::result::Result<std::path::PathBuf, ViuError> {
    let (mut tmpfile, path) = tempfile::Builder::new()
        .prefix(".tmp.viuer.")
        .rand_bytes(1)
        .tempfile()?
        .keep()?;

    tmpfile.write_all(buf).unwrap();
    tmpfile.flush().unwrap();
    Ok(path)
}

// Delete any images that intersect with the cursor. Used to improve performance, i.e
// to "forget" old images.
pub fn kitty_delete() -> ViuResult {
    print!("\x1b_Ga=d,d=C\x1b\\");
    Ok(())
}
