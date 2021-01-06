use crate::error::{ViuError, ViuResult};
use crate::printer::{find_best_fit, Printer};
use crate::Config;
use failure::{format_err, Error};
use image::DynamicImage;
use image::GenericImageView;
use lazy_static::lazy_static;
use std::env;
use std::io::Read;

pub type MResult<T> = Result<T, Error>;
trait WithRaw {
    fn with_raw(&self, fun: impl FnOnce(&[u8]) -> ViuResult) -> ViuResult;
}

trait ImgSize {
    fn size(&self) -> MResult<(usize, usize)>;
}

impl ImgSize for DynamicImage {
    fn size(&self) -> MResult<(usize, usize)> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        Ok((width, height))
    }
}

pub struct SixelPrinter {}

impl WithRaw for image::DynamicImage {
    fn with_raw(&self, fun: impl FnOnce(&[u8]) -> ViuResult) -> ViuResult {
        fun(self.as_bytes())
    }
}

impl Printer for SixelPrinter {
    fn print(&self, img: &image::DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
        print_sixel(img, config).map(|_| -> (u32, u32) { (img.width(), img.height()) })
    }
}

fn print_sixel(img: &image::DynamicImage, config: &Config) -> ViuResult {
    use sixel::encoder::{Encoder, QuickFrameBuilder};
    use sixel::optflags::EncodePolicy;

    let (xpix, ypix) = img.size()?;

    let (w, h) = find_best_fit(&img, config.width, config.height);
    let rgba = img.to_rgba8();
    let raw = rgba.as_raw();

    let bob = w as usize;
    let larry = h as usize;
    print!("w : {}, h: {}", bob, larry);

    // img.with_raw(move |raw| -> ViuResult {
    let sixfail = |e| format_err!("Sixel failed with: {:?}", e);
    let encoder = Encoder::new().map_err(sixfail)?;

    encoder
        .set_encode_policy(EncodePolicy::Fast)
        .map_err(sixfail)?;

    let frame = QuickFrameBuilder::new()
        .width(xpix)
        .height(ypix)
        .format(sixel_sys::PixelFormat::RGBA8888)
        .pixels(raw.to_vec());

    encoder.encode_bytes(frame).map_err(sixfail)?;

    // No end of line printed by encoder
    println!("");
    println!("");

    Ok(())
    // })
}

impl std::convert::From<failure::Error> for crate::error::ViuError {
    fn from(e: failure::Error) -> Self {
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
    /// The Kitty graphics protocol is not supported.
    None,
    /// Kitty is running locally, data can be shared through a file.
    Local,
    /// Kitty is not running locally, data has to be sent through escape codes.
    Remote,
}
///TODO check for sixel support on windows
#[cfg(windows)]
fn check_sixel_support() -> SixelSupport {
    SixelSupport::None;
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

    term_info.c_iflag &= !(ISTRIP);
    term_info.c_iflag &= !(INLCR);
    term_info.c_iflag &= !(ICRNL);
    term_info.c_iflag &= !(IGNCR);
    term_info.c_iflag &= !(IXOFF);

    term_info.c_lflag &= !(ECHO);
    term_info.c_lflag &= !(ICANON);

    tcsetattr(file_descriptor, TCSANOW, &mut term_info)?;
    write("/dev/tty", "\x1b[0c")?;
    let mut std_in_buffer: [u8; 256] = [0; 256];
    let size_read = stdin().read(&mut std_in_buffer)?;
    let mut found_sixel_support = false;
    for i in 0..size_read {
        if std_in_buffer[i] == 52 {
            found_sixel_support = true;
            break;
        }
    }
    term_info.c_iflag = old_iflag;
    term_info.c_lflag = old_lflag;
    tcsetattr(file_descriptor, TCSANOW, &mut term_info)?;
    return Ok(if found_sixel_support {
        SixelSupport::Local
    } else {
        SixelSupport::None
    });
}

// // Check if Sixel protocol can be used
#[cfg(unix)]
fn check_sixel_support() -> SixelSupport {
    use SixelSupport::{Local, None};
    match env::var("TERM").unwrap_or(String::from("None")).as_str() {
        "mlterm" => Local,
        "yaft-256color" => Local,
        "xterm-256color" => xterm_check_sixel_support().unwrap_or(None),
        _ => match env::var("TERM_PROGRAM")
            .unwrap_or(String::from("None"))
            .as_str()
        {
            "MacTerm" => Local,
            _ => None,
        },
    }
}

///Ignore this test because it
///only passes on systems with
///sixel support
#[test]
#[ignore]
fn pixel_support() {
    match check_sixel_support() {
        SixelSupport::Local => (),
        SixelSupport::None => assert!(false),
        SixelSupport::Remote => (),
    }
}
