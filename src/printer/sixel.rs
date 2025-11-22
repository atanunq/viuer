use crate::error::ViuResult;
use crate::printer::{adjust_offset, find_best_fit, Printer, ReadKey};
use crate::Config;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use sixel_rs::encoder::{Encoder, QuickFrameBuilder};
use sixel_rs::optflags::EncodePolicy;
use std::io::Write;

#[derive(Debug)]
pub struct SixelPrinter;

impl Printer for SixelPrinter {
    fn print(
        &self,
        _stdin: &impl ReadKey,
        stdout: &mut impl Write,
        img: &DynamicImage,
        config: &Config,
    ) -> ViuResult<(u32, u32)> {
        let (w, h) = find_best_fit(img, config.width, config.height);

        //TODO: the max 1000 width is an xterm bug workaround, other terminals may not be affected
        let resized_img =
            img.resize_exact(std::cmp::min(6 * w, 1000), 12 * h, FilterType::Triangle);

        let (width, height) = resized_img.dimensions();

        let rgba = resized_img.to_rgba8();
        let raw = rgba.as_raw();

        adjust_offset(stdout, config)?;

        let encoder = Encoder::new()?;

        encoder.set_encode_policy(EncodePolicy::Fast)?;

        let frame = QuickFrameBuilder::new()
            .width(width as usize)
            .height(height as usize)
            .format(sixel_rs::sys::PixelFormat::RGBA8888)
            .pixels(raw.to_vec());

        encoder.encode_bytes(frame)?;

        Ok((w, h))
    }
}
