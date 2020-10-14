use crate::error::{ViuError, ViuResult};
use crate::printer::Printer;
use console::{Key, Term};
use image::GenericImageView;
use std::io::Write;

pub struct KittyPrinter {}

impl Printer for KittyPrinter {
    fn print(img: &image::DynamicImage, config: &crate::Config) -> ViuResult {
        match has_kitty_support() {
            KittySupport::None => {
                //give up, print blocks
                todo!()
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

#[derive(PartialEq)]
pub enum KittySupport {
    // The Kitty graphics protocol cannot be used
    None,
    // Kitty is running locally, data can be shared through a file
    Local,
    // Kitty is not running locally, data has to be sent through escape codes
    Remote,
}

///
pub fn has_kitty_support() -> KittySupport {
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
    let (mut tmpfile, path) = tempfile::Builder::new()
        .prefix(".tmp.viuer.")
        .rand_bytes(1)
        .tempfile()?
        .keep()?;

    // create a 1x1 image and write it to the temp file
    let x = image::RgbaImage::new(1, 1);
    tmpfile.write_all(x.as_raw())?;
    tmpfile.flush()?;

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

    Err(ViuError::Kitty(response))
}

// Print with kitty graphics protocol through a temp file
// TODO: make sure this is fine when many files are displayed consecutively
fn print_local(img: &image::DynamicImage, config: &crate::Config) -> ViuResult {
    let rgba = img.to_rgba();
    let raw_img = rgba.as_raw();

    let (mut tmpfile, path) = tempfile::Builder::new()
        .prefix(".tmp.viuer.")
        .rand_bytes(1)
        .tempfile()?
        .keep()?;

    tmpfile.write_all(raw_img).unwrap();
    tmpfile.flush().unwrap();

    // print!("\x1b_Ga=P,x={},y={}\x1b\\", config.x, config.y);

    print!(
        "\x1b_Gf=32,s={},v={},c=100,r=30,a=T,t=t,X={},Y={};{}\x1b\\",
        img.width(),
        img.height(),
        config.x,
        config.y,
        base64::encode(path.to_str().unwrap())
    );
    println!();
    std::io::stdout().flush().unwrap();
    Ok(())
}
//TODO default_features false of console
