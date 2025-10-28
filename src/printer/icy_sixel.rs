use super::{adjust_offset, find_best_fit, Printer};
use icy_sixel::sixel_string;
use image::{imageops::FilterType, GenericImageView};

pub struct IcySixelPrinter;

impl Printer for IcySixelPrinter {
    fn print(
        &self,
        stdout: &mut impl std::io::Write,
        img: &image::DynamicImage,
        config: &crate::Config,
    ) -> crate::ViuResult<(u32, u32)> {
        let (w, h) = find_best_fit(img, config.width, config.height);

        //TODO: the max 1000 width is an xterm bug workaround, other terminals may not be affected
        let resized_img =
            img.resize_exact(std::cmp::min(6 * w, 1000), 12 * h, FilterType::Triangle);

        let (width, height) = resized_img.dimensions();

        let rgba = resized_img.to_rgba8();
        let raw = rgba.as_raw();

        adjust_offset(stdout, config)?;

        match sixel_string(
            raw,
            width as i32,
            height as i32,
            icy_sixel::PixelFormat::RGBA8888,
            icy_sixel::DiffusionMethod::Auto,
            icy_sixel::MethodForLargest::Auto,
            icy_sixel::MethodForRep::Auto,
            icy_sixel::Quality::AUTO,
        ) {
            Ok(output) => {
                write!(stdout, "{output}")?;
                stdout.flush()?;
                Ok((w, h))
            }
            Err(error) => Err(crate::ViuError::IcySixelError(format!("{error}"))),
        }
    }
}
