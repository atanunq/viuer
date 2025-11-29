use crate::error::{ViuError, ViuResult};
use crate::printer::{adjust_offset, find_best_fit, Printer, ReadKey};
use crate::utils::terminal_size;
use crate::Config;
use base64::{engine::general_purpose, Engine};
use console::{Key, Term};
use std::io::Write;
use std::io::{Error, ErrorKind};
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
        let result = match get_kitty_support() {
            KittySupport::None => Err(ViuError::KittyNotSupported),
            KittySupport::Local => {
                // print from file
                print_local(stdin, stdout, img, config)
            }
            KittySupport::Remote => {
                // print through escape codes
                print_remote(stdin, stdout, img, config)
            }
        }?;

        print_newline(stdout, config, result.0)?;

        Ok(result)
    }

    // TODO: guess_format() here in order to treat PNGs specially (f=100).
    // Also, maybe get channel count and use f=24 or f=32 accordingly.
    // fn print_from_file(&self, filename: &str, config: &Config) -> ViuResult<(u32, u32)> {}
}

/// The cursor is pushed to the next line by Kitty if the image reaches the terminal's boundary.
/// We must do it only if the image is smaller, otherwise we end up with a blank line.
/// See <https://github.com/atanunq/viuer/pull/90#discussion_r2557013728>
///
/// Could be done with a cursor check through `crossterm::cursor::position`,
/// but that alone doesn't justify enabling the `events` feature.
fn print_newline(stdout: &mut impl Write, config: &Config, width: u32) -> ViuResult {
    let (term_w, _) = terminal_size();
    if config.x + (width as u16) < term_w {
        writeln!(stdout)?;
    }

    Ok(())
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
    let mut stdout = std::io::stdout();
    let term = Term::stdout();

    // first check if kitty protocol base-line(inline images / remote) is available
    if has_remote_support(&term, &mut stdout).is_ok() {
        // then test if the current terminal supports reading from a file(local) (for example this is not possible via ssh)
        if has_local_support(&term, &mut stdout).is_ok() {
            return KittySupport::Local;
        }

        return KittySupport::Remote;
    }

    KittySupport::None
}

/// Close the temporary file that was created, filtering out [`NotFound`](ErrorKind::NotFound) errors.
fn close_tmp_file(temp_file: NamedTempFile) -> ViuResult {
    // Explicitly clean up when finished with the file because destructor, OS and Kitty are not deterministic.
    if let Err(err) = temp_file.close() {
        // Proper Kitty terminals *will delete* the file after fully reading it, if it is in a known temporary directory
        // so we dont want to error if the file does not exist anymore
        if err.kind() != ErrorKind::NotFound {
            return Err(err.into());
        }
    }

    Ok(())
}

/// Send & Wait for the DSR(Device Status Report) query.
fn wait_for_dsr(stdin: &impl ReadKey, stdout: &mut impl Write) -> ViuResult {
    write!(stdout, "\x1b[5n")?;
    stdout.flush()?;

    let mut response = Vec::new();

    // assign it once instead of having to allocate a vector with static content in each loop
    let end_seq = Key::UnknownEscSeq(vec!['[', '0', 'n']);

    while let Ok(key) = stdin.read_key() {
        // The response will end with Esc('x1b'), followed by Backslash('\').
        // Also, break if the Unknown key is found, which is returned when we're not in a tty
        // https://sw.kovidgoyal.net/kitty/graphics-protocol/#display-images-on-screen
        let should_break = key == end_seq || key == Key::Unknown;
        response.push(key);
        if should_break {
            break;
        }
    }

    if response.len() == 1 && response.last().unwrap() == &end_seq {
        return Ok(());
    }

    Err(ViuError::KittyResponse(response))
}

/// Query the terminal whether it can display inline images (the kitty image protocol base-line)
fn has_remote_support(stdin: &impl ReadKey, stdout: &mut impl Write) -> ViuResult {
    // send the query
    write!(
        stdout,
        // First query send a query for kitty image protocol
        "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\",
    )?;
    // Then, send "primary device attributes" as a fall-back if kitty protocol is not supported.
    // This is practically implemented in all terminals and prevents a infinite loop if kitty is not supported.
    write!(stdout, "\x1b[c")?;
    stdout.flush()?;

    let mut response = Vec::new();

    // determine if we had the "primary device attributes" reply, as otherwise "c" *could* be part of another query response beforehand
    let mut had_pda: bool = false;

    while let Ok(key) = stdin.read_key() {
        // this sequenece is also called "CSI ? 6" in "Terminal Response" at https://vt100.net/docs/vt510-rm/DA1.html
        // Though it seems the "6" (and "4") is not actually required if the vt is at a different level
        // meaning if we test explicitly for "6" it would never exit
        if let Key::UnknownEscSeq(esc_seq) = &key {
            if esc_seq.starts_with(&['[', '?']) {
                had_pda = true;
            }
        }

        // The "primary device attributes" response will end with a "c" character
        // see "Terminal Response" at https://vt100.net/docs/vt510-rm/DA1.html
        // Alternatively, terminate on unknown keys, this could for example happen in cargo test with a `console::Term` read_key, for some reason
        let should_break = (had_pda && key == Key::Char('c')) || key == Key::Unknown;

        response.push(key);

        if should_break {
            break;
        }
    }

    // The Graphics query response
    let expected = [
        Key::UnknownEscSeq(['_'].into()),
        Key::Char('G'),
        Key::Char('i'),
        Key::Char('='),
        Key::Char('3'),
        Key::Char('1'),
        Key::Char(';'),
        Key::Char('O'),
        Key::Char('K'),
        Key::UnknownEscSeq(vec!['\\']),
    ];

    // The Graphics query and the device attributes response could theoretically be in any order
    // but most terminals will reply in a FIFO order
    if response.len() >= expected.len() && response[..expected.len()] == expected {
        return Ok(());
    }

    Err(ViuError::KittyResponse(response))
}

/// Query the terminal whether it can display an image from a file.
///
/// Note that this function expects at least the base-line kitty support.
/// If this function is run first, this could lead to a infinite loop if kitty is not supported.
/// Run [`has_remote_support`] first and only run this function if that returns `Ok`.
fn has_local_support(stdin: &impl ReadKey, stdout: &mut impl Write) -> ViuResult {
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

    let mut response = Vec::new();

    while let Ok(key) = stdin.read_key() {
        // The response will end with Esc('x1b'), followed by Backslash('\').
        // Also, break if the Unknown key is found, which is returned when we're not in a tty
        // https://sw.kovidgoyal.net/kitty/graphics-protocol/#display-images-on-screen
        let should_break = key == Key::UnknownEscSeq(vec!['\\']) || key == Key::Unknown;
        response.push(key);
        if should_break {
            break;
        }
    }

    close_tmp_file(temp_file)?;

    // The Graphics query response
    let expected = [
        Key::UnknownEscSeq(['_'].into()),
        Key::Char('G'),
        Key::Char('i'),
        Key::Char('='),
        Key::Char('3'),
        Key::Char('1'),
        Key::Char(';'),
        Key::Char('O'),
        Key::Char('K'),
        Key::UnknownEscSeq(vec!['\\']),
    ];

    // The Graphics query and the device attributes response could theoretically be in any order
    // but most terminals will reply in a FIFO order
    if response.len() >= expected.len() && response[..expected.len()] == expected {
        return Ok(());
    }

    Err(ViuError::KittyResponse(response))
}

/// Print with kitty graphics protocol through a temp file
// TODO: try with kitty's supported compression
fn print_local(
    stdin: &impl ReadKey,
    stdout: &mut impl Write,
    img: &image::DynamicImage,
    config: &Config,
) -> ViuResult<(u32, u32)> {
    let rgba = img.to_rgba8();
    let raw_img = rgba.as_raw();
    let temp_file = store_in_tmp_file(raw_img)?;

    adjust_offset(stdout, config)?;

    // get the desired width and height
    let (w, h) = find_best_fit(
        img,
        config.width,
        config.height,
        config.preserve_aspect_ratio,
    );

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
    stdout.flush()?;

    // prevent race condition of removing the file before the terminal is finished reading it.
    wait_for_dsr(stdin, stdout)?;

    close_tmp_file(temp_file)?;

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

    let (w, h) = find_best_fit(
        img,
        config.width,
        config.height,
        config.preserve_aspect_ratio,
    );

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

        let test_data = [Key::UnknownEscSeq(vec!['[', '0', 'n'])];
        let test_response = TestKeys::new(&test_data);

        assert_eq!(
            print_local(&test_response, &mut vec, &img, &config).unwrap(),
            (40, 13)
        );
        let result = std::str::from_utf8(&vec).unwrap();

        assert!(result.starts_with("\x1b[4;5H\x1b_Gf=32,s=40,v=25,c=40,r=13,a=T,t=t;"));
        assert!(result.ends_with("\x1b\\\x1b[5n"));
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
            "\x1b[6;3H\x1b_Gf=32,a=T,t=d,s=1,v=2,c=1,r=1,m=1;AAAAAAIEBgg=\x1b\\"
        );
        assert!(test_response.reached_end());
    }

    #[test]
    fn test_kitty_supported_remote_and_local() {
        // output collected on kitty 0.42.2

        // test kitty protocol support
        let mut stdout = Vec::new();

        let test_data = [
            Key::UnknownEscSeq(vec!['_']),
            Key::Char('G'),
            Key::Char('i'),
            Key::Char('='),
            Key::Char('3'),
            Key::Char('1'),
            Key::Char(';'),
            Key::Char('O'),
            Key::Char('K'),
            Key::UnknownEscSeq(vec!['\\']),
            Key::UnknownEscSeq(vec!['[', '?', '6']),
            Key::Char('2'),
            Key::Char(';'),
            Key::Char('5'),
            Key::Char('2'),
            Key::Char(';'),
            Key::Char('c'),
        ];
        let test_response = TestKeys::new(&test_data);

        has_remote_support(&test_response, &mut stdout).unwrap();
        let result = std::str::from_utf8(&stdout).unwrap();

        assert_eq!(result, "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c");
        assert!(test_response.reached_end());

        stdout.clear();

        // test kitty local protocol support
        let mut stdout = Vec::new();

        let test_data = [
            Key::UnknownEscSeq(vec!['_']),
            Key::Char('G'),
            Key::Char('i'),
            Key::Char('='),
            Key::Char('3'),
            Key::Char('1'),
            Key::Char(';'),
            Key::Char('O'),
            Key::Char('K'),
            Key::UnknownEscSeq(vec!['\\']),
        ];
        let test_response = TestKeys::new(&test_data);

        has_local_support(&test_response, &mut stdout).unwrap();
        let result = std::str::from_utf8(&stdout).unwrap();

        assert!(result.starts_with("\x1b_Gi=31,s=1,v=1,a=q,t=t;"));
        assert!(result.ends_with("\x1b\\"));
        assert!(test_response.reached_end());
    }

    #[test]
    fn test_remote_support_tmux() {
        // output collected on tmux 3.5_a (kitty & Konsole)

        // test kitty protocol support
        let mut stdout = Vec::new();

        let test_data = [
            Key::UnknownEscSeq(vec!['[', '?', '1']),
            Key::Char(';'),
            Key::Char('2'),
            Key::Char(';'),
            Key::Char('4'),
            Key::Char('c'),
        ];
        let test_response = TestKeys::new(&test_data);

        has_remote_support(&test_response, &mut stdout).unwrap_err();
        let result = std::str::from_utf8(&stdout).unwrap();

        assert_eq!(result, "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c");
        assert!(test_response.reached_end());
    }

    #[test]
    fn test_kitty_supported_but_not_local() {
        // output collected on konsole 25.08.1

        // test kitty protocol support
        let mut stdout = Vec::new();

        let test_data = [
            Key::UnknownEscSeq(vec!['_']),
            Key::Char('G'),
            Key::Char('i'),
            Key::Char('='),
            Key::Char('3'),
            Key::Char('1'),
            Key::Char(';'),
            Key::Char('O'),
            Key::Char('K'),
            Key::UnknownEscSeq(vec!['\\']),
            Key::UnknownEscSeq(vec!['[', '?', '6']),
            Key::Char('2'),
            Key::Char(';'),
            Key::Char('1'),
            Key::Char(';'),
            Key::Char('4'),
            Key::Char('c'),
        ];
        let test_response = TestKeys::new(&test_data);

        has_remote_support(&test_response, &mut stdout).unwrap();
        let result = std::str::from_utf8(&stdout).unwrap();

        assert_eq!(result, "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c");
        assert!(test_response.reached_end());

        stdout.clear();

        // test kitty local protocol support
        let mut stdout = Vec::new();

        let test_data = [
            Key::UnknownEscSeq(vec!['_']),
            Key::Char('G'),
            Key::Char('i'),
            Key::Char('='),
            Key::Char('3'),
            Key::Char('1'),
            Key::Char(';'),
            Key::Char('E'),
            Key::Char('N'),
            Key::Char('O'),
            Key::Char('T'),
            Key::Char('S'),
            Key::Char('U'),
            Key::Char('P'),
            Key::Char('P'),
            Key::Char('O'),
            Key::Char('R'),
            Key::Char('T'),
            Key::Char('E'),
            Key::Char('D'),
            Key::Char(':'),
            Key::UnknownEscSeq(vec!['\\']),
        ];
        let test_response = TestKeys::new(&test_data);

        has_local_support(&test_response, &mut stdout).unwrap_err();
        let result = std::str::from_utf8(&stdout).unwrap();

        assert!(result.starts_with("\x1b_Gi=31,s=1,v=1,a=q,t=t;"));
        assert!(result.ends_with("\x1b\\"));
        assert!(test_response.reached_end());
    }

    #[test]
    fn test_no_kitty_support() {
        let mut stdout = Vec::new();

        // only the "primary device attributes"
        let test_data = [Key::UnknownEscSeq(['[', '?', '6'].into()), Key::Char('c')];
        let test_response = TestKeys::new(&test_data);

        has_remote_support(&test_response, &mut stdout).unwrap_err();
        let result = std::str::from_utf8(&stdout).unwrap();

        assert_eq!(result, "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c");
        assert!(test_response.reached_end());
    }
}
