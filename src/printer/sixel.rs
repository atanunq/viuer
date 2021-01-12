use crate::error::ViuResult;
use crate::printer::{adjust_offset, Printer};
use crate::Config;
use console::{Key, Term};
use image::{DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use sixel::encoder::{Encoder, QuickFrameBuilder};
use sixel::optflags::EncodePolicy;
use std::io::Write;

pub struct SixelPrinter {}

lazy_static! {
    static ref SIXEL_SUPPORT: bool = check_sixel_support();
}

/// Returns the terminal's support for Sixel.
pub fn is_sixel_supported() -> bool {
    *SIXEL_SUPPORT
}

impl Printer for SixelPrinter {
    fn print(&self, img: &DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
        let (width, height) = img.dimensions();

        let rgba = img.to_rgba8();
        let raw = rgba.as_raw();

        let mut stdout = std::io::stdout();
        adjust_offset(&mut stdout, config)?;

        //TODO: get the desired width and height
        // let (w, h) = find_best_fit(&img, config.width, config.height);

        let encoder = Encoder::new()?;

        encoder.set_encode_policy(EncodePolicy::Fast)?;

        let frame = QuickFrameBuilder::new()
            .width(width as usize)
            .height(height as usize)
            .format(sixel_sys::PixelFormat::RGBA8888)
            .pixels(raw.to_vec());

        encoder.encode_bytes(frame)?;

        Ok((width, height))
    }
}

// Check if Sixel is within the terminal's attributes
// see https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Sixel-Graphics
// and https://vt100.net/docs/vt510-rm/DA1.html
fn check_device_attrs() -> ViuResult<bool> {
    print!("\x1b[c");
    std::io::stdout().flush()?;

    let term = Term::stdout();
    let mut response = String::new();

    while let Ok(key) = term.read_key() {
        if let Key::Char(c) = key {
            response.push(c);
            if c == 'c' {
                break;
            }
        }
    }

    Ok(response.contains(";4;") || response.contains(";4c"))
}

// Check if Sixel protocol can be used
fn check_sixel_support() -> bool {
    if let Ok(term) = std::env::var("TERM") {
        match term.as_str() {
            "mlterm" | "yaft-256color" => return true,
            "st-256color" | "xterm" | "xterm-256color" => return check_device_attrs().is_ok(),
            _ => {
                if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
                    return term_program == "MacTerm";
                }
            }
        }
    }
    false
}
