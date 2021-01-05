use crate::error::{ViuError, ViuResult};
use crate::printer::{adjust_offset, find_best_fit, Printer};
use crate::Config;
use console::{Key, Term};
use image::GenericImageView;
use lazy_static::lazy_static;
use std::io::Write;
use std::io::{ ErrorKind};
use image::DynamicImage;
use failure::{Error, format_err};

pub type MResult<T> = Result<T, Error>;
trait WithRaw {
    fn with_raw(&self,
                fun: impl FnOnce(&[u8]) -> ViuResult)
                -> ViuResult;
}


trait ImgSize {
    fn size(&self) -> MResult<(usize, usize)>;
}

impl ImgSize for DynamicImage {
    fn size(&self) -> MResult<(usize, usize)> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        Ok((width, height))
    }
}

pub struct SixelPrinter {

}

impl WithRaw for image::DynamicImage {
    fn with_raw(&self,
        fun: impl FnOnce(&[u8]) -> ViuResult)
        -> ViuResult {
    fun(self.as_bytes())
}
}


impl Printer for SixelPrinter {
    fn print(&self, img: &image::DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
        print_sixel(img).map(|_| -> (u32, u32) {
            (img.width(),img.height())
        })
    }

}

pub fn print_sixel(img: &(impl WithRaw+ImgSize)) -> ViuResult {
    use sixel::encoder::{Encoder, QuickFrameBuilder};
    use sixel::optflags::EncodePolicy;

    let (xpix, ypix) = img.size()?;

    img.with_raw(move |raw| -> ViuResult {
        let sixfail = |e| format_err!("Sixel failed with: {:?}", e);
        let encoder = Encoder::new()
            .map_err(sixfail)?;

        encoder.set_encode_policy(EncodePolicy::Fast)
            .map_err(sixfail)?;
        
        let frame = QuickFrameBuilder::new()
            .width(xpix)
            .height(ypix)
            .format(sixel_sys::PixelFormat::RGBA8888)
            .pixels(raw.to_vec());

        encoder.encode_bytes(frame)
            .map_err(sixfail)?;

        // No end of line printed by encoder
        println!("");
        println!("");

        Ok(())
    })
}

impl std::convert::From<failure::Error> for crate::error::ViuError {
    fn from(e: failure::Error) -> Self {
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
    /// The Kitty graphics protocol is not supported.
    None,
    /// Kitty is running locally, data can be shared through a file.
    Local,
    /// Kitty is not running locally, data has to be sent through escape codes.
    Remote,
}

// Check if Kitty protocol can be used
fn check_sixel_support() -> SixelSupport {
    //TODO check for actual support TASK
    // if let Ok(term) = std::env::var("TERM") {
    //     if term.contains("kitty") {
    //         if has_local_support().is_ok() {
    //             return KittySupport::Local;
    //         } else {
    //             return KittySupport::Remote;
    //         }
    //     }
    // }
    // KittySupport::None
}

// Query the terminal whether it can display an image from a file
// fn has_local_support() -> ViuResult {
    //TODO do this
    // create a temp file that will hold a 1x1 image
    // let x = image::RgbaImage::new(1, 1);
    // let raw_img = x.as_raw();
    // let path = store_in_tmp_file(raw_img)?;

    // // send the query
    // print!(
    //     // t=t tells Kitty it's reading from a temp file and will delete if afterwards
    //     "\x1b_Gi=31,s=1,v=1,a=q,t=t;{}\x1b\\",
    //     base64::encode(path.to_str().ok_or_else(|| std::io::Error::new(
    //         std::io::ErrorKind::Other,
    //         "Could not convert path to &str"
    //     ))?)
    // );
    // std::io::stdout().flush()?;

    // // collect Kitty's response after the query
    // let term = Term::stdout();
    // let mut response = Vec::new();

    // // TODO: could use a queue of length 3
    // while let Ok(key) = term.read_key() {
    //     // the response will end with Esc('x1b'), followed by Backslash('\')
    //     let should_break = key == Key::UnknownEscSeq(vec!['\\']);
    //     response.push(key);
    //     if should_break {
    //         break;
    //     }
    // }

    // // Kitty response should end with these 3 Keys if it was successful
    // let expected = [
    //     Key::Char('O'),
    //     Key::Char('K'),
    //     Key::UnknownEscSeq(vec!['\\']),
    // ];

    // if response.len() >= expected.len() && response[response.len() - 3..] == expected {
    //     return Ok(());
    // }

    // Err(ViuError::KittyResponse(response))
// }

// Print with kitty graphics protocol through a temp file
// TODO: try with kitty's supported compression
// fn print_local(img: &image::DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {

    // let rgba = img.to_rgba8();
    // let raw_img = rgba.as_raw();
    // let path = store_in_tmp_file(raw_img)?;

    // let mut stdout = std::io::stdout();
    // adjust_offset(&mut stdout, config)?;

    // // get the desired width and height
    // let (w, h) = find_best_fit(&img, config.width, config.height);

    // write!(
    //     stdout,
    //     "\x1b_Gf=32,s={},v={},c={},r={},a=T,t=t;{}\x1b\\",
    //     img.width(),
    //     img.height(),
    //     w,
    //     h,
    //     base64::encode(path.to_str().ok_or_else(|| ViuError::IO(Error::new(
    //         ErrorKind::Other,
    //         "Could not convert path to &str"
    //     )))?)
    // )?;
    // writeln!(stdout)?;
    // stdout.flush()?;

    // Ok((w, h))
// }

// // Print with escape codes
// // TODO: try compression
// fn print_remote(img: &image::DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
//     let rgba = img.to_rgba8();
//     let raw = rgba.as_raw();
//     let encoded = base64::encode(raw);
//     let mut iter = encoded.chars().peekable();

//     let mut stdout = std::io::stdout();
//     adjust_offset(&mut stdout, config)?;

//     let (w, h) = find_best_fit(&img, config.width, config.height);

//     let first_chunk: String = iter.by_ref().take(4096).collect();

//     // write the first chunk, which describes the image
//     write!(
//         stdout,
//         "\x1b_Gf=32,a=T,t=d,s={},v={},c={},r={},m=1;{}\x1b\\",
//         img.width(),
//         img.height(),
//         w,
//         h,
//         first_chunk
//     )?;

//     // write all the chunks, each containing 4096 bytes of data
//     while iter.peek().is_some() {
//         let chunk: String = iter.by_ref().take(4096).collect();
//         let m = if iter.peek().is_some() { 1 } else { 0 };
//         write!(stdout, "\x1b_Gm={};{}\x1b\\", m, chunk)?;
//     }
//     writeln!(stdout)?;
//     stdout.flush()?;
//     Ok((w, h))
// }

// // Create a file in temporary dir and write the byte slice to it.
// fn store_in_tmp_file(buf: &[u8]) -> std::result::Result<std::path::PathBuf, ViuError> {
//     let (mut tmpfile, path) = tempfile::Builder::new()
//         .prefix(".tmp.viuer.")
//         .rand_bytes(1)
//         .tempfile()?
//         // Since the file is persisted, the user is responsible for deleting it afterwards. However,
//         // Kitty does this automatically after printing from a temp file.
//         .keep()?;

//     tmpfile.write_all(buf)?;
//     tmpfile.flush()?;
//     Ok(path)
// }
