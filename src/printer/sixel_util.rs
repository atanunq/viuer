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
fn check_device_attrs() -> ViuResult<bool> {
    let mut term = Term::stdout();

    write!(&mut term, "\x1b[c")?;
    term.flush()?;

    let mut response = String::new();

    while let Ok(key) = term.read_key() {
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
    if let Ok(term) = std::env::var("TERM") {
        match term.as_str() {
            "yaft-256color" | "eat-truecolor" => {
                return true;
            }
            _ => {
                if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
                    if term_program == "MacTerm" {
                        return true;
                    }
                }
            }
        }
    }
    check_device_attrs().unwrap_or(false)
}
