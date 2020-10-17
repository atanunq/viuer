use crate::error::{ViuError, ViuResult};
use crate::printer::Printer;
use console::{Key, Term};
use crossterm::cursor::{MoveRight, MoveTo, MoveToPreviousLine};
use crossterm::execute;
use image::GenericImageView;
use lazy_static::lazy_static;
use std::io::Write;

pub struct KittyPrinter {}

lazy_static! {
    pub static ref KITTY_SUPPORT: KittySupport = has_kitty_support();
}

impl Printer for KittyPrinter {
    fn print(img: &image::DynamicImage, config: &crate::Config) -> ViuResult {
        match *KITTY_SUPPORT {
            KittySupport::None => {
                //give up, print blocks
                Err(ViuError::KittyNotSupported)
            }
            KittySupport::Local => {
                //print from file
                print_local(img, config)
            }
            KittySupport::Remote => {
                //print through escape codes
                todo!()
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
///
pub enum KittySupport {
    /// The Kitty graphics protocol cannot be used.
    None,
    /// Kitty is running locally, data can be shared through a file.
    Local,
    /// Kitty is not running locally, data has to be sent through escape codes.
    Remote,
}

///
fn has_kitty_support() -> KittySupport {
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
    // adjust y offset
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
    // TODO: create a calc_dimensions function that does the same without actually modifying an image.
    //  Also, the h returend is 2* what we want, because of blocks' height
    let (w, h) = crate::resize(img, config.width, config.height).dimensions();

    print!(
        "\x1b_Gf=32,s={},v={},c={},r={},a=T,t=t,X={},Y={};{}\x1b\\",
        img.width(),
        img.height(),
        w,
        h / 2,
        config.x,
        config.y,
        base64::encode(path.to_str().unwrap())
    );
    println!();
    stdout.flush().unwrap();

    Ok(())
}
//TODO default_features false of console

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
