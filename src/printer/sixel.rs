use crate::error::{ViuError, ViuResult};
use crate::printer::{Printer};
use crate::Config;
use image::GenericImageView;
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
    fn print(&self, img: &image::DynamicImage, _config: &Config) -> ViuResult<(u32, u32)> {
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