use crate::error::{ViuError, ViuResult};
use crate::printer::{adjust_offset, find_best_fit, Printer, ReadKey};
use crate::Config;
use base64::{engine::general_purpose, Engine};
use console::{Key, Term};
use std::io::Error;
use std::io::Write;
use std::sync::LazyLock;
use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct KittyPrinter;

const TEMP_FILE_PREFIX: &str = ".tty-graphics-protocol.viuer.";
static KITTY_SUPPORT: LazyLock<KittySupport> = LazyLock::new(check_kitty_support);

/// Returns the terminal's support for the Kitty graphics protocol.
pub fn get_kitty_support() -> KittySupport {
    *KITTY_SUPPORT
}

impl Printer for KittyPrinter {
    fn print(
        &self,
        stdin: &impl ReadKey,
        stdout: &mut impl Write,
        img: &image::DynamicImage,
        config: &Config,
    ) -> ViuResult<(u32, u32)> {
        match get_kitty_support() {
            KittySupport::None => Err(ViuError::KittyNotSupported),
            KittySupport::Local => {
                // print from file
                print_local(stdin, stdout, img, config)
            }
            KittySupport::Remote => {
                // print through escape codes
                print_remote(stdin, stdout, img, config)
            }
        }
    }

    // TODO: guess_format() here in order to treat PNGs specially (f=100).
    // Also, maybe get channel count and use f=24 or f=32 accordingly.
    // fn print_from_file(&self, filename: &str, config: &Config) -> ViuResult<(u32, u32)> {}
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// The extend to which the Kitty graphics protocol can be used.
pub enum KittySupport {
    /// The Kitty graphics protocol is not supported.
    None,
    /// Kitty is running locally, data can be shared through a file.
    Local,
    /// Kitty is not running locally, data has to be sent through escape codes.
    Remote,
}

/// Check if Kitty protocol can be used
fn check_kitty_support() -> KittySupport {
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("kitty") || term.contains("ghostty") {
            let mut stdout = std::io::stdout();
            let stdin = Term::stdout();
            if has_local_support(&mut stdout, &stdin).is_ok() {
                return KittySupport::Local;
            }

            return KittySupport::Remote;
        }
    }
    KittySupport::None
}

/// Query the terminal whether it can display an image from a file
fn has_local_support(stdout: &mut impl Write, stdin: &impl ReadKey) -> ViuResult {
    // create a temp file that will hold a 1x1 image
    let x = image::RgbaImage::new(1, 1);
    let raw_img = x.as_raw();
    let temp_file = store_in_tmp_file(raw_img)?;

    // send the query
    write!(
        stdout,
        // t=t tells Kitty it's reading from a temp file and will attempt to delete if afterwards
        "\x1b_Gi=31,s=1,v=1,a=q,t=t;{}\x1b\\",
        general_purpose::STANDARD.encode(
            temp_file
                .path()
                .to_str()
                .ok_or_else(|| ViuError::Io(Error::other("Could not convert path to &str")))?
        )
    )?;
    stdout.flush()?;

    // collect Kitty's response after the query
    let mut response = Vec::new();

    while let Ok(key) = stdin.read_key() {
        // The response will end with Esc('x1b'), followed by Backslash('\').
        // Also, break if the Unknown key is found, which is returned when we're not in a tty
        let should_break = key == Key::UnknownEscSeq(vec!['\\']) || key == Key::Unknown;
        response.push(key);
        if should_break {
            break;
        }
    }

    // Explicitly clean up when finished with the file because destructor, OS and Kitty are not deterministic.
    temp_file.close()?;

    // Kitty response should end with these 3 Keys if it was successful
    let expected = [
        Key::Char('O'),
        Key::Char('K'),
        Key::UnknownEscSeq(vec!['\\']),
    ];

    if response.len() >= expected.len() && response[response.len() - 3..] == expected {
        return Ok(());
    }

    Err(ViuError::KittyResponse(response))
}

/// Print with kitty graphics protocol through a temp file
// TODO: try with kitty's supported compression
fn print_local(
    _stdin: &impl ReadKey,
    stdout: &mut impl Write,
    img: &image::DynamicImage,
    config: &Config,
) -> ViuResult<(u32, u32)> {
    let rgba = img.to_rgba8();
    let raw_img = rgba.as_raw();
    let temp_file = store_in_tmp_file(raw_img)?;

    adjust_offset(stdout, config)?;

    // get the desired width and height
    let (w, h) = find_best_fit(img, config.width, config.height);

    write!(
        stdout,
        "\x1b_Gf=32,s={},v={},c={},r={},a=T,t=t;{}\x1b\\",
        img.width(),
        img.height(),
        w,
        h,
        general_purpose::STANDARD.encode(
            temp_file
                .path()
                .to_str()
                .ok_or_else(|| ViuError::Io(Error::other("Could not convert path to &str")))?
        )
    )?;
    writeln!(stdout)?;
    stdout.flush()?;

    // Explicitly clean up when finished with the file because destructor, OS and Kitty are not deterministic.
    temp_file.close()?;

    Ok((w, h))
}

/// Print with escape codes
// TODO: try compression
fn print_remote(
    _stdin: &impl ReadKey,
    stdout: &mut impl Write,
    img: &image::DynamicImage,
    config: &Config,
) -> ViuResult<(u32, u32)> {
    let rgba = img.to_rgba8();
    let raw = rgba.as_raw();
    let encoded = general_purpose::STANDARD.encode(raw);
    let mut iter = encoded.chars().peekable();

    adjust_offset(stdout, config)?;

    let (w, h) = find_best_fit(img, config.width, config.height);

    let first_chunk: String = iter.by_ref().take(4096).collect();

    // write the first chunk, which describes the image
    write!(
        stdout,
        "\x1b_Gf=32,a=T,t=d,s={},v={},c={},r={},m=1;{}\x1b\\",
        img.width(),
        img.height(),
        w,
        h,
        first_chunk
    )?;

    // write all the chunks, each containing 4096 bytes of data
    while iter.peek().is_some() {
        let chunk: String = iter.by_ref().take(4096).collect();
        let m = if iter.peek().is_some() { 1 } else { 0 };
        write!(stdout, "\x1b_Gm={};{}\x1b\\", m, chunk)?;
    }
    writeln!(stdout)?;
    stdout.flush()?;
    Ok((w, h))
}

/// Create a file in temporary dir and write the byte slice to it.
/// The NamedTempFile will be deleted once it goes out of scope.
fn store_in_tmp_file(buf: &[u8]) -> std::result::Result<NamedTempFile, ViuError> {
    let mut tmpfile = tempfile::Builder::new()
        .prefix(TEMP_FILE_PREFIX)
        .rand_bytes(1)
        .tempfile()?;

    tmpfile.write_all(buf)?;
    tmpfile.flush()?;
    Ok(tmpfile)
}

#[cfg(test)]
mod tests {
    use crate::printer::TestKeys;

    use super::*;
    use image::{DynamicImage, GenericImage};

    #[test]
    fn test_print_local() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::new(40, 25));
        let config = Config {
            x: 4,
            y: 3,
            ..Default::default()
        };

        let mut vec = Vec::new();

        let test_data = [];
        let test_response = TestKeys::new(&test_data);

        assert_eq!(
            print_local(&test_response, &mut vec, &img, &config).unwrap(),
            (40, 13)
        );
        let result = std::str::from_utf8(&vec).unwrap();

        assert!(result.starts_with("\x1b[4;5H\x1b_Gf=32,s=40,v=25,c=40,r=13,a=T,t=t;"));
        assert!(result.ends_with("\x1b\\\n"));
        assert!(test_response.reached_end());
    }

    #[test]
    fn test_print_remote() {
        let mut img = DynamicImage::ImageRgba8(image::RgbaImage::new(1, 2));
        img.put_pixel(0, 1, image::Rgba([2, 4, 6, 8]));

        let config = Config {
            x: 2,
            y: 5,
            ..Default::default()
        };

        let mut vec = Vec::new();

        let test_data = [];
        let test_response = TestKeys::new(&test_data);

        assert_eq!(
            print_remote(&test_response, &mut vec, &img, &config).unwrap(),
            (1, 1)
        );
        let result = std::str::from_utf8(&vec).unwrap();

        assert_eq!(
            result,
            "\x1b[6;3H\x1b_Gf=32,a=T,t=d,s=1,v=2,c=1,r=1,m=1;AAAAAAIEBgg=\x1b\\\n"
        );
        assert!(test_response.reached_end());
    }

    #[test]
    fn test_kitty_protocol_supported() {
        // test kitty protocol support
        let mut stdout = Vec::new();

        // data returned by the terminal from the query
        // Captured from kitty 0.42.2
        let test_stdin_data = [
            // the following response indicated a successful graphical request
            Key::UnknownEscSeq(['_'].into()),
            Key::Char('G'),
            Key::Char('i'),
            Key::Char('='),
            Key::Char('3'),
            Key::Char('1'),
            Key::Char(';'),
            Key::Char('O'),
            Key::Char('K'),
            Key::UnknownEscSeq(['\\'].into()),
        ];
        let test_stdin = TestKeys::new(&test_stdin_data);

        has_local_support(&mut stdout, &test_stdin).unwrap();
        let result = std::str::from_utf8(&stdout).unwrap();

        // assert_eq!(result, "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c");
        assert!(result.starts_with("\x1b_Gi=31,s=1,v=1,a=q,t=t;"));
        assert!(result.ends_with("\x1b\\"));
        assert!(test_stdin.reached_end());
    }
}
