use crate::error::{ViuError, ViuResult};
use crate::printer::Printer;
use crate::Config;
use image::DynamicImage;
use image::GenericImageView;
use lazy_static::lazy_static;
use std::env;
use std::io::{Read, Write};

pub struct SixelPrinter {}

impl Printer for SixelPrinter {
    fn print(&self, img: &DynamicImage, _config: &Config) -> ViuResult<(u32, u32)> {
        print_sixel(img)
    }

    fn print_from_file(&self, filename: &str, _config: &Config) -> ViuResult<(u32, u32)> {
        print_sixel_from_file(filename)
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

    Ok((x_pixles, y_pixels))
}

/// Print sixel from a file.
/// This will block the thread.
/// If the file is a gif, this will block the
/// thread indefinitely.
pub fn print_sixel_from_file(filename: &str) -> ViuResult<(u32, u32)> {
    use sixel::encoder::Encoder;
    use sixel::optflags::EncodePolicy;

    let encoder = Encoder::new()?;

    encoder.set_encode_policy(EncodePolicy::Fast)?;
    encoder.encode_file(std::path::Path::new(filename))?;

    let mut stdout = std::io::stdout();
    stdout.flush()?;
    Ok((0, 0))
}

impl std::convert::From<sixel::status::Error> for crate::error::ViuError {
    fn from(e: sixel::status::Error) -> Self {
        ViuError::SixelError(e)
    }
}

lazy_static! {
    static ref SIXEL_SUPPORT: SixelSupport = check_sixel_support();
}

/// Returns the terminal's support for the Kitty graphics protocol.
pub fn get_sixel_support() -> SixelSupport {
    *SIXEL_SUPPORT
}

#[derive(PartialEq, Copy, Clone)]
/// The extend to which the Kitty graphics protocol can be used.
pub enum SixelSupport {
    /// The Sixel graphics protocol is not supported.
    None,
    /// The Sixel graphics protocol is supported.
    Supported,
}
///TODO check for sixel support on windows
#[cfg(windows)]
fn xterm_check_sixel_support() -> Result<SixelSupport, std::io::Error> {
    SixelSupport::None
}

#[cfg(unix)]
fn xterm_check_sixel_support() -> Result<SixelSupport, std::io::Error> {
    use std::fs::write;
    use std::io::stdin;
    use termios::*;
    //STDOUT_FILENO
    let file_descriptor = 1;
    let mut term_info = Termios::from_fd(file_descriptor)?;
    let old_iflag = term_info.c_iflag;
    let old_lflag = term_info.c_lflag;
    //setup the terminal so that it will send the device attributes
    //to stdin rather than writing them to the screen
    term_info.c_iflag &= !(ISTRIP);
    term_info.c_iflag &= !(INLCR);
    term_info.c_iflag &= !(ICRNL);
    term_info.c_iflag &= !(IGNCR);
    term_info.c_iflag &= !(IXOFF);

    term_info.c_lflag &= !(ECHO);
    term_info.c_lflag &= !(ICANON);

    tcsetattr(file_descriptor, TCSANOW, &mut term_info)?;

    //Send Device Attributes
    // see https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_
    write("/dev/tty", "\x1b[0c")?;
    let mut std_in_buffer: [u8; 256] = [0; 256];
    let size_read = stdin().read(&mut std_in_buffer)?;
    let mut found_sixel_support = false;
    for i in 0..size_read {
        //52 is ascii for 4
        if std_in_buffer[i] == 52 {
            found_sixel_support = true;
            break;
        }
    }
    term_info.c_iflag = old_iflag;
    term_info.c_lflag = old_lflag;
    tcsetattr(file_descriptor, TCSANOW, &mut term_info)?;
    Ok(if found_sixel_support {
        SixelSupport::Supported
    } else {
        SixelSupport::None
    })
}

// // Check if Sixel protocol can be used
fn check_sixel_support() -> SixelSupport {
    use SixelSupport::{None, Supported};
    match env::var("TERM").unwrap_or(String::from("None")).as_str() {
        "mlterm" => Supported,
        "yaft-256color" => Supported,
        "xterm-256color" => xterm_check_sixel_support().unwrap_or(None),
        _ => match env::var("TERM_PROGRAM")
            .unwrap_or(String::from("None"))
            .as_str()
        {
            "MacTerm" => Supported,
            _ => None,
        },
    }
}

///Ignore this test because it
///only passes on systems with
///sixel support
#[test]
#[ignore]
fn sixel_support() {
    match check_sixel_support() {
        SixelSupport::Supported => (),
        SixelSupport::None => assert!(false),
    }
}
