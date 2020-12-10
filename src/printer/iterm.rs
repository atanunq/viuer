use crate::error::{ViuError, ViuResult};
use crate::printer::{adjust_offset, find_best_fit, Printer};
use crate::Config;
use lazy_static::lazy_static;
use std::io::Write;
use std::io::{BufReader, Error, ErrorKind, Read};

#[allow(non_camel_case_types)]
pub struct iTermPrinter {}

lazy_static! {
    static ref ITERM_SUPPORT: bool = check_iterm_support();
}

/// Returns the terminal's support for the iTerm graphics protocol.
pub fn is_iterm_supported() -> bool {
    *ITERM_SUPPORT
}

impl Printer for iTermPrinter {
    fn print(&self, img: &image::DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
        let temp_file = tempfile::Builder::new()
            .prefix(".tmp.viuer.")
            .suffix(".jpeg")
            .rand_bytes(1)
            .tempfile()?;

        match temp_file.path().to_str() {
            Some(path) => {
                img.save(path)?;
                let (w, h) = self.print_from_file(&path, config)?;

                Ok((w, h))
            }
            None => Err(ViuError::IO(Error::new(
                ErrorKind::Other,
                "Could not convert path to &str",
            ))),
        }
    }

    fn print_from_file(&self, filename: &str, config: &Config) -> ViuResult<(u32, u32)> {
        let file = std::fs::File::open(filename)?;
        let file_len = file.metadata()?.len();

        // load the file content
        let mut buf_reader = BufReader::new(file);
        let mut file_content = Vec::with_capacity(file_len as usize);
        buf_reader.read_to_end(&mut file_content)?;

        // decode the image from the file content
        let img = image::load_from_memory(&file_content[..])?;

        let mut stdout = std::io::stdout();
        adjust_offset(&mut stdout, config)?;

        let (w, h) = find_best_fit(&img, config.width, config.height);

        writeln!(
            stdout,
            "\x1b]1337;File=inline=1;preserveAspectRatio=1;size={};width={};height={}:{}\x07",
            file_len,
            w,
            h,
            base64::encode(file_content)
        )?;
        stdout.flush()?;

        Ok((w, h))
    }
}

// Check if the iTerm protocol can be used
fn check_iterm_support() -> bool {
    if let Ok(term) = std::env::var("TERM_PROGRAM") {
        if term.contains("iTerm") {
            return true;
        }
    }
    false
}
