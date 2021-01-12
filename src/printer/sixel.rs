use crate::error::ViuResult;
use crate::printer::Printer;
use crate::Config;
use console::{Key, Term};
use image::{DynamicImage, GenericImageView};
use lazy_static::lazy_static;
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
    fn print(&self, img: &DynamicImage, _config: &Config) -> ViuResult<(u32, u32)> {
        print_sixel(img)
    }
}

fn print_sixel(img: &image::DynamicImage) -> ViuResult<(u32, u32)> {
    use sixel::encoder::{Encoder, QuickFrameBuilder};
    use sixel::optflags::EncodePolicy;

    let (x_pixles, y_pixels) = img.dimensions();

    let rgba = img.to_rgba8();
    let raw = rgba.as_raw();

    let encoder = Encoder::new()?;

    encoder.set_encode_policy(EncodePolicy::Fast)?;

    let frame = QuickFrameBuilder::new()
        .width(x_pixles as usize)
        .height(y_pixels as usize)
        .format(sixel_sys::PixelFormat::RGBA8888)
        .pixels(raw.to_vec());

    encoder.encode_bytes(frame)?;

    // No end of line printed by encoder
    let mut stdout = std::io::stdout();
    stdout.flush()?;

    let y_pixel_size = get_pixel_size();
    let small_y_pixels = y_pixels as u16;
    Ok((
        x_pixles,
        if y_pixel_size == 0 {
            5000
        } else {
            (small_y_pixels / y_pixel_size + 1) as u32
        },
    ))
}

#[cfg(windows)]
fn get_pixel_size() -> u16 {
    0
}

#[cfg(unix)]
#[derive(Debug)]
#[repr(C)]
struct winsize {
    ws_row: libc::c_ushort,
    ws_col: libc::c_ushort,
    ws_xpixel: libc::c_ushort,
    ws_ypixel: libc::c_ushort,
}

#[cfg(unix)]
fn get_pixel_size() -> u16 {
    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    const TIOCGWINSZ: libc::c_ulong = 0x40087468;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const TIOCGWINSZ: libc::c_ulong = 0x5413;
    let size_out = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        if libc::ioctl(1, TIOCGWINSZ, &size_out) != 0 {
            return 0;
        }
    }
    if size_out.ws_ypixel == 0 || size_out.ws_row == 0 {
        return 0;
    }
    size_out.ws_ypixel / size_out.ws_row
}

// Check if Sixel is within the terminal's attributes
// see https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Sixel-Graphics
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
