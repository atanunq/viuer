use crate::error::ViuResult;
use crate::printer::{adjust_offset, find_best_fit, Printer, ReadKey};
use crate::{Config, ViuError};
use base64::{engine::general_purpose, Engine};
use console::{Key, Term};
use image::{DynamicImage, GenericImageView, ImageEncoder};
use std::io::Write;
use std::sync::LazyLock;

#[cfg(feature = "print-file")]
use std::{
    io::{BufReader, Read},
    path::Path,
};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct iTermPrinter;

static ITERM_SUPPORT: LazyLock<bool> = LazyLock::new(check_iterm_support);

/// Returns the terminal's support for the iTerm graphics protocol.
pub fn is_iterm_supported() -> bool {
    *ITERM_SUPPORT
}

impl Printer for iTermPrinter {
    fn print(
        &self,
        _stdin: &impl ReadKey,
        stdout: &mut impl Write,
        img: &DynamicImage,
        config: &Config,
    ) -> ViuResult<(u32, u32)> {
        // DEBUG: REMOVE THIS BEFORE MERGE
        eprintln!("using iterm");
        let (width, height) = img.dimensions();

        // Transform the dynamic image to a PNG which can be given directly to iTerm
        let mut png_bytes: Vec<u8> = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png_bytes).write_image(
            img.as_bytes(),
            width,
            height,
            img.color().into(),
        )?;

        print_buffer(stdout, img, &png_bytes[..], config)
    }

    #[cfg(feature = "print-file")]
    fn print_from_file<P: AsRef<Path>>(
        &self,
        _stdin: &impl ReadKey,
        stdout: &mut impl Write,
        filename: P,
        config: &Config,
    ) -> ViuResult<(u32, u32)> {
        let file = std::fs::File::open(filename)?;

        // load the file content
        let mut buf_reader = BufReader::new(file);
        let mut file_content = Vec::new();
        buf_reader.read_to_end(&mut file_content)?;

        let img = image::load_from_memory(&file_content[..])?;
        print_buffer(stdout, &img, &file_content[..], config)
    }
}

/// This function requires both a DynamicImage, which is used to calculate dimensions,
/// and it's raw representation as a file, because that's the data iTerm needs to display it.
fn print_buffer(
    stdout: &mut impl Write,
    img: &DynamicImage,
    img_content: &[u8],
    config: &Config,
) -> ViuResult<(u32, u32)> {
    adjust_offset(stdout, config)?;

    let (w, h) = find_best_fit(img, config.width, config.height);

    writeln!(
        stdout,
        "\x1b]1337;File=inline=1;preserveAspectRatio=1;size={};width={};height={}:{}\x07",
        img_content.len(),
        w,
        h,
        general_purpose::STANDARD.encode(img_content)
    )?;
    stdout.flush()?;

    Ok((w, h))
}

const ITERM_CAP_REPLY_SIZE: usize = "1337;Capabilities=".len();

/// Check if the terminal supports "iterm2 inline image protocol" by querying capabilities.
///
/// This function is based on what is written in <https://iterm2.com/feature-reporting/> and <https://gitlab.com/gnachman/iterm2/-/issues/10236>.
fn has_iterm_support_capabilities(stdout: &mut impl Write, stdin: &impl ReadKey) -> ViuResult {
    // send the query
    write!(
        stdout,
        // Query iterm2 capabilities https://iterm2.com/feature-reporting/
        "\x1b]1337;Capabilities\x1b\\"
    )?;
    // send extra "Device Status Report (DSR)" which practically all terminals respond to, to avoid infinitely blocking if not replied to the query above
    // see https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_
    write!(stdout, "\x1b[5n")?;

    stdout.flush()?;

    let mut response = Vec::new();

    let end_seq = Key::UnknownEscSeq(vec!['[', '0', 'n']);

    while let Ok(key) = stdin.read_key() {
        // The response will end with the reply to "\x1b[5n", which is "\x1b[0n"
        // Also, break if the Unknown key is found, which is returned when we're not in a tty
        let should_break = key == end_seq || key == Key::Unknown;
        response.push(key);
        if should_break {
            break;
        }
    }

    // DEBUG: REMOVE THIS BEFORE MERGE
    eprintln!("Iterm2 Response Capabilities: {:#?}", response);

    // no response to the "Capabilities" query, or pre-maturely ended without the DSR
    if response.last() != Some(&end_seq) || response.len() < ITERM_CAP_REPLY_SIZE + 1/* ST */ + 1
    /* END SEQ */
    {
        return Err(ViuError::ItermResponse(response));
    }

    // The response format *should* be "\x1b]1337;Capabilities=" followed by FEATURES and finally end with "\x1b\\"(ESC \).
    // FEATURES have the format for Capital letter, followed by none or more lowercase letteres, followed by none or more digits
    // repeated without any delimiter.
    // We only care about the "FILE" feature, which has simply just "F"

    // remove the "intro" and the ST(stop) and the DSR from the search
    let trunc_response = &response[ITERM_CAP_REPLY_SIZE..=response.len() - 2];

    if trunc_response.contains(&Key::Char('F')) {
        return Ok(());
    }

    Err(ViuError::ItermResponse(response))
}

const ITERM_CELL_REPLY_SIZE: usize = "1337;ReportCellSize=".len();

/// Check if the terminal liekly supports "iterm2 inline image protocol" by querying ReportCellSize.
///
/// This function is based on what is written in <https://iterm2.com/documentation-escape-codes.html#report-cell-size> and <https://github.com/atanunq/viuer/pull/88>.
fn has_iterm_support_reportcellsize(stdout: &mut impl Write, stdin: &impl ReadKey) -> ViuResult {
    // send the query
    write!(
        stdout,
        // Query iterm2 ReportCellSize https://iterm2.com/documentation-escape-codes.html#report-cell-size
        "\x1b]1337;ReportCellSize\x1b\\"
    )?;
    // send extra "Device Status Report (DSR)" which practically all terminals respond to, to avoid infinitely blocking if not replied to the query above
    // see https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_
    write!(stdout, "\x1b[5n")?;

    stdout.flush()?;

    let mut response = Vec::new();

    let end_seq = Key::UnknownEscSeq(vec!['[', '0', 'n']);

    while let Ok(key) = stdin.read_key() {
        // The response will end with the reply to "\x1b[5n", which is "\x1b[0n"
        // Also, break if the Unknown key is found, which is returned when we're not in a tty
        let should_break = key == end_seq || key == Key::Unknown;
        response.push(key);
        if should_break {
            break;
        }
    }

    // DEBUG: REMOVE THIS BEFORE MERGE
    eprintln!("Iterm2 Response ReportCellSize: {:#?}", response);

    // no response to the "Capabilities" query, or pre-maturely ended without the DSR
    if response.last() != Some(&end_seq) || response.len() < ITERM_CELL_REPLY_SIZE + 1/* ST */ + 1
    /* END SEQ */
    {
        return Err(ViuError::ItermResponse(response));
    }

    // Check if the response we got actually contains the correct response (we only check the echo name)
    const EXPECTED_RESPONSE: &str = "ReportCellSize";

    let mut chars = EXPECTED_RESPONSE.chars().peekable();
    let mut consumed: usize = 0;

    // Check each key in the response and try to exhaust the "chars" iter, which will indicate the full response is available
    for key in &response {
        if chars.peek().map(|v| Key::Char(*v)).as_ref() == Some(key) {
            let _ = chars.next();
            consumed += 1;
        } else if consumed != 0 {
            // we hit something not in the correct sequence, so we break here
            // for example we hit "ReportVariable" where we want to stop at "V" instead of trying to continue
            break;
        }
    }

    if chars.next().is_none() {
        return Ok(());
    }

    Err(ViuError::ItermResponse(response))
}

/// Check if the iTerm protocol can be used
fn check_iterm_support() -> bool {
    let mut stdout = std::io::stdout();
    let term = Term::stdout();
    if has_iterm_support_capabilities(&mut stdout, &term).is_ok() {
        // DEBUG: REMOVE THIS BEFORE MERGE
        eprintln!("Capabilities OK");
        return true;
    }
    if has_iterm_support_reportcellsize(&mut stdout, &term).is_ok() {
        // DEBUG: REMOVE THIS BEFORE MERGE
        eprintln!("ReportCellSize OK");
        return true;
    }

    if let Ok(term) = std::env::var("TERM_PROGRAM") {
        if term.contains("iTerm")
            || term.contains("WezTerm")
            || term.contains("mintty")
            || term.contains("rio")
            || term.contains("WarpTerminal")
        {
            return true;
        }
    }
    if let Ok(lc_term) = std::env::var("LC_TERMINAL") {
        if lc_term.contains("iTerm")
            || lc_term.contains("WezTerm")
            || lc_term.contains("mintty")
            || lc_term.contains("rio")
        {
            return true;
        }
    }
    // Konsole does not have "TERM_PROGRAM" and only has "TERM=xterm-256color", which is too generic
    // but in exchange, there is the following Konsole-only environment variable with which we can detect it
    if let Ok(version) = std::env::var("KONSOLE_VERSION") {
        if !version.is_empty() {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::printer::TestKeys;

    use super::*;
    use image::GenericImage;

    #[test]
    fn test_print_e2e() {
        let mut img = DynamicImage::ImageRgba8(image::RgbaImage::new(2, 3));
        img.put_pixel(1, 2, image::Rgba([2, 4, 6, 8]));

        let config = Config {
            x: 4,
            y: 3,
            ..Default::default()
        };
        let mut vec = Vec::new();

        let stdin = TestKeys::new(&[]);

        assert_eq!(
            iTermPrinter.print(&stdin, &mut vec, &img, &config).unwrap(),
            (2, 2)
        );
        assert_eq!(std::str::from_utf8(&vec).unwrap(), "\x1b[4;5H\x1b]1337;File=inline=1;preserveAspectRatio=1;size=95;width=2;height=2:iVBORw0KGgoAAAANSUhEUgAAAAIAAAADCAYAAAC56t6BAAAAJklEQVR4AQEbAOT/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACBAYIAEMAFdTlTsEAAAAASUVORK5CYII=\x07\n");
    }

    #[test]
    fn capabilities_iterm2_should_reply() {
        let mut vec = Vec::new();

        let test_data = [
            // intro
            Key::UnknownEscSeq(vec![']']), // TODO: is this actually returning that, or differently? i dont know how "console" handles OSC
            Key::Char('1'),
            Key::Char('3'),
            Key::Char('3'),
            Key::Char('7'),
            Key::Char(';'),
            Key::Char('C'),
            Key::Char('a'),
            Key::Char('p'),
            Key::Char('a'),
            Key::Char('b'),
            Key::Char('i'),
            Key::Char('l'),
            Key::Char('i'),
            Key::Char('t'),
            Key::Char('i'),
            Key::Char('e'),
            Key::Char('s'),
            Key::Char('='),
            // actual features (not actual features except "F")
            Key::Char('T'),
            Key::Char('3'),
            Key::Char('L'),
            Key::Char('r'),
            // maybe more
            Key::Char('F'),
            Key::Char('S'),
            Key::Char('x'),
            // ST
            Key::UnknownEscSeq(vec!['\\']),
            // DSR
            Key::UnknownEscSeq(vec!['[', '0', 'n']),
        ];
        let test_response = TestKeys::new(&test_data);

        assert_eq!(
            has_iterm_support_capabilities(&mut vec, &test_response).unwrap(),
            ()
        );
        let stdout = std::str::from_utf8(&vec).unwrap();

        assert_eq!(stdout, "\x1b]1337;Capabilities\x1b\\\x1b[5n");
        assert!(test_response.reached_end());
    }

    #[test]
    fn capabilites_should_handle_no_reply() {
        let mut vec = Vec::new();

        let test_data = [
            // DSR
            Key::UnknownEscSeq(['[', '0', 'n'].into()),
        ];
        let test_response = TestKeys::new(&test_data);

        assert!(has_iterm_support_capabilities(&mut vec, &test_response).is_err());
        let stdout = std::str::from_utf8(&vec).unwrap();

        assert_eq!(stdout, "\x1b]1337;Capabilities\x1b\\\x1b[5n");
        assert!(test_response.reached_end());
    }

    #[test]
    fn reportcellsize_iterm2_should_reply() {
        // output captured on konsole 25.08.1
        let mut vec = Vec::new();

        let test_data = [
            // intro
            Key::UnknownEscSeq(vec![']']),
            Key::Char('1'),
            Key::Char('3'),
            Key::Char('3'),
            Key::Char('7'),
            Key::Char(';'),
            Key::Char('R'),
            Key::Char('e'),
            Key::Char('p'),
            Key::Char('o'),
            Key::Char('r'),
            Key::Char('t'),
            Key::Char('C'),
            Key::Char('e'),
            Key::Char('l'),
            Key::Char('l'),
            Key::Char('S'),
            Key::Char('i'),
            Key::Char('z'),
            Key::Char('e'),
            Key::Char('='),
            // actual response
            Key::Char('1'),
            Key::Char('5'),
            Key::Char('.'),
            Key::Char('0'),
            Key::Char(';'),
            Key::Char('1'),
            Key::Char('.'),
            Key::Char('0'),
            // BEL / Bell
            Key::Char('\x07'),
            // DSR
            Key::UnknownEscSeq(vec!['[', '0', 'n']),
        ];
        let test_response = TestKeys::new(&test_data);

        assert_eq!(
            has_iterm_support_reportcellsize(&mut vec, &test_response).unwrap(),
            ()
        );
        let stdout = std::str::from_utf8(&vec).unwrap();

        assert_eq!(stdout, "\x1b]1337;ReportCellSize\x1b\\\x1b[5n");
        assert!(test_response.reached_end());
    }

    #[test]
    fn reportcellsize_should_handle_no_reply() {
        let mut vec = Vec::new();

        let test_data = [
            // DSR
            Key::UnknownEscSeq(['[', '0', 'n'].into()),
        ];
        let test_response = TestKeys::new(&test_data);

        assert!(has_iterm_support_reportcellsize(&mut vec, &test_response).is_err());
        let stdout = std::str::from_utf8(&vec).unwrap();

        assert_eq!(stdout, "\x1b]1337;ReportCellSize\x1b\\\x1b[5n");
        assert!(test_response.reached_end());
    }
}
