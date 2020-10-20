use std::env;

const DEFAULT_TERM_SIZE: (u16, u16) = (80, 24);

pub fn truecolor_available() -> bool {
    if let Ok(value) = env::var("COLORTERM") {
        value.contains("truecolor") || value.contains("24bit")
    } else {
        false
    }
}

/// Try to get the terminal size. If unsuccessful, fallback to a default (80x24).
///
/// Uses [crossterm::terminal::size].
/// ## Example
/// The example below prints "img.jpg" with dimensions 80x40 in the center of the terminal.
/// ```no_run
/// use viuer::{Config, print_from_file, terminal_size};
///
/// let (term_width, term_height) = terminal_size();
/// // Set desired image dimensions
/// let width = 80;
/// let height = 40;
///
/// let config = Config {
///     x: (term_width - width) / 2,
///     y: (term_height - height) as i16 / 2,
///     width: Some(width as u32),
///     height: Some(height as u32),
///     ..Default::default()
/// };
/// print_from_file("img.jpg", &config).expect("Image printing failed.");
/// ```
#[cfg(not(test))]
pub fn terminal_size() -> (u16, u16) {
    match crossterm::terminal::size() {
        Ok(s) => s,
        Err(_) => DEFAULT_TERM_SIZE,
    }
}

// Return a constant when running the tests
#[cfg(test)]
pub fn terminal_size() -> (u16, u16) {
    DEFAULT_TERM_SIZE
}

/// Given width & height of an image, scale the size so that it can fit within given bounds
/// while preserving aspect ratio. Will scale both up and down.
pub fn fit_dimensions(width: u32, height: u32, bound_width: u32, bound_height: u32) -> (u32, u32) {
    let ratio = width * bound_height;
    let nratio = bound_width * height;

    let use_width = nratio <= ratio;
    let intermediate = if use_width {
        height * bound_width / width
    } else {
        width * bound_height / height
    };

    if use_width {
        (bound_width, intermediate)
    } else {
        (intermediate, bound_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truecolor() {
        env::set_var("COLORTERM", "truecolor");
        assert!(truecolor_available());
        env::set_var("COLORTERM", "");
        assert!(!truecolor_available());
    }

    #[test]
    fn test_fit() {
        // ratio 1:1
        assert_eq!((40, 40), fit_dimensions(100, 100, 40, 50));
        assert_eq!((30, 30), fit_dimensions(100, 100, 40, 30));
        // ratio 3:2
        assert_eq!((30, 20), fit_dimensions(240, 160, 30, 100));
        // ratio 5:7
        assert_eq!((100, 140), fit_dimensions(300, 420, 320, 140));
        // ratio 4:3
        assert_eq!((32, 24), fit_dimensions(4, 3, 80, 24));
    }
}
