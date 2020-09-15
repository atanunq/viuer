use std::env;

pub fn truecolor_available() -> bool {
    if let Ok(value) = env::var("COLORTERM") {
        value.contains("truecolor") || value.contains("24bit")
    } else {
        false
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
}
