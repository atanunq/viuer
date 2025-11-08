use crate::printer::ReadKey;
use crate::ViuResult;
use console::{Key, Term};
use std::io::Write;
use std::sync::LazyLock;

static SIXEL_SUPPORT: LazyLock<bool> = LazyLock::new(check_sixel_support);

/// Returns the terminal's support for Sixel.
pub fn is_sixel_supported() -> bool {
    *SIXEL_SUPPORT
}

// Check if Sixel is within the terminal's attributes
// see https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Sixel-Graphics
// and https://vt100.net/docs/vt510-rm/DA1.html
fn check_device_attrs(stdout: &mut impl Write, stdin: &impl ReadKey) -> ViuResult<bool> {
    write!(stdout, "\x1b[c")?;
    stdout.flush()?;

    let mut response = String::new();

    while let Ok(key) = stdin.read_key() {
        // exit on first "Unknown" key as we know that this is not a proper response anymore
        if key == Key::Unknown {
            break;
        }

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
    let mut stdout = std::io::stdout();
    let stdin = Term::stdout();
    check_device_attrs(&mut stdout, &stdin).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use console::Key;

    use crate::printer::{sixel_util::check_device_attrs, TestKeys};

    #[test]
    fn should_detect_device_attrs() {
        // test kitty protocol support
        let mut stdout = Vec::new();

        // data returned by the terminal from the query
        // Captured from Konsole 25.08.1
        let test_stdin_data = [
            // CSI ? DEVCLASS
            Key::UnknownEscSeq(['[', '?', '6'].into()),
            // DEVCLASS-2
            // in this case this corresponds to "?62;", or "VT220"
            Key::Char('2'),
            Key::Char(';'),
            // 132 columns
            Key::Char('1'),
            Key::Char(';'),
            // sixel support
            Key::Char('4'),
            // response end
            Key::Char('c'),
        ];
        let test_stdin = TestKeys::new(&test_stdin_data);

        check_device_attrs(&mut stdout, &test_stdin).unwrap();
        let result = std::str::from_utf8(&stdout).unwrap();

        assert_eq!(result, "\x1b[c");
        assert!(test_stdin.reached_end());
    }
}
