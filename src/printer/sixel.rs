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
        if y_pixel_size <= 0 {
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
    if size_out.ws_ypixel <= 0 || size_out.ws_row <= 0 {
        return 0;
    }
    size_out.ws_ypixel / size_out.ws_row
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

// Parsing the escape code sequence
// see
// http://www.ecma-international.org/publications/files/ECMA-ST/Ecma-048.pdf
// section 5.4.2
// about parsing Parameter string format
// and section
// and   8.3.16  for the definition of CSI (spoiler alert, it's Escape [)
// Note: In ECMA-048 docs,  05/11 is the same as 0x5B which is the hex ascii code for [
// And see
// https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_
// CSI Ps c  Send Device Attributes (Primary DA).
enum XTERMSupportParserState {
    ExpectCSIESC,
    ExpectCSIOpenBracket,
    ExpectQuestionMark,
    ParseParameter,
    ParseParameterNotFour,
    ParseParameterMightBeFour,
    InvalidState,
    FoundFour,
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
    let mut state = XTERMSupportParserState::ExpectCSIESC;
    for i in 0..size_read {
        let current_char = std_in_buffer[i];
        use XTERMSupportParserState::{
            ExpectCSIESC, ExpectCSIOpenBracket, ExpectQuestionMark, FoundFour, InvalidState,
            ParseParameter, ParseParameterMightBeFour, ParseParameterNotFour,
        };

        match state {
            ExpectCSIESC => {
                //ascii for ESC
                if current_char != 27 {
                    state = InvalidState;
                    break;
                }
                state = ExpectCSIOpenBracket;
            }
            ExpectCSIOpenBracket => {
                //ascii for [
                if current_char != 91 {
                    state = InvalidState;
                    break;
                }
                state = ExpectQuestionMark;
            }
            ExpectQuestionMark => {
                //ascii for ?
                if current_char != 63 {
                    state = InvalidState;
                    break;
                }
                state = ParseParameter;
            }
            //ascii for 4
            ParseParameter => {
                state = if current_char != 52 {
                    ParseParameterNotFour
                } else {
                    ParseParameterMightBeFour
                }
            }
            //ascii for ; which is the separator
            ParseParameterNotFour => {
                state = if current_char != 59 {
                    ParseParameterNotFour
                } else {
                    ParseParameter
                }
            }
            //59 is ascii for ; which is the separator and
            // 99 is ascii for c which marks the end of the phrase
            ParseParameterMightBeFour => {
                if current_char == 59 || current_char == 99 {
                    state = FoundFour;
                    break;
                }
                state = ParseParameterNotFour;
            }
            InvalidState => break,
            FoundFour => break,
        }
    }
    term_info.c_iflag = old_iflag;
    term_info.c_lflag = old_lflag;
    tcsetattr(file_descriptor, TCSANOW, &mut term_info)?;

    Ok(if let XTERMSupportParserState::FoundFour = state {
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
        "st-256color" => xterm_check_sixel_support().unwrap_or(None),
        "xterm" => xterm_check_sixel_support().unwrap_or(None),
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
