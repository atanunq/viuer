use console::{Key, Term};
use lazy_static::lazy_static;
use std::io::Write;
use crate::ViuResult;

lazy_static! {
    static ref SIXEL_SUPPORT: bool = check_sixel_support();
}

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
            "mlterm" | "yaft-256color" | "foot" | "foot-extra" | "eat-truecolor" => return true,
            "st-256color" | "xterm" | "xterm-256color" => {
                return check_device_attrs().unwrap_or(false)
            }
            _ => {
                if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
                    return term_program == "MacTerm";
                }
            }
        }
    }

    // Windows terminal.
    #[cfg(feature = "icy_sixel")]
    if let Ok(_guid) = std::env::var("WT_SESSION") {
        return true;
    }

    false
}
