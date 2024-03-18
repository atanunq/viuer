use image::{GenericImage, ImageBuffer, Pixel, Rgba, RgbaImage};
use termcolor::Color;
use crate::printer::block::maskers::{ALL_BLOCK_ELEMENTS, CharMasker, Masker};

pub const SUBPIXEL64: usize = 8; // the ratio of subpixellized pixels to pixel

pub const SUBPIXEL64_ROWS: usize = SUBPIXEL64 * 2;
pub const SUBPIXEL64_COLUMNS: usize = SUBPIXEL64;

pub struct Mask {
    pub char: char,
    pub mask: [[bool; SUBPIXEL64_COLUMNS]; SUBPIXEL64_ROWS]
}

impl Mask {
    pub fn new(masker: CharMasker) -> Mask {
        let mut mask = [[false; SUBPIXEL64_COLUMNS]; SUBPIXEL64_ROWS];
        for row in 0..SUBPIXEL64_ROWS {
            for column in 0..SUBPIXEL64_COLUMNS {
                mask[row][column] = masker.mask(row, column);
            }
        }
        Mask { char: masker.0, mask }
    }
}

pub fn get_all_masks() -> Vec<Mask> {
    ALL_BLOCK_ELEMENTS.chars()
        .map(|c| CharMasker(c))
        .map(|cm| Mask::new(cm))
        .collect()
}

pub fn get_mask_colors(img: &RgbaImage, mask: &Mask) -> Option<(Rgba<u8>, Rgba<u8>)> {
    let mut fg_colors = vec![];
    let mut bg_colors = vec![];
    let (width, height) = img.dimensions();
    for row in 0..SUBPIXEL64_ROWS as u32 {
        for column in 0..SUBPIXEL64_COLUMNS as u32 {
            let col = if row >= height || column >= width {
                Rgba([0, 0, 0, 0])
            } else {
                *img.get_pixel(column, row)
            };
            if mask.mask[row as usize][column as usize] {
                if !fg_colors.contains(&col) { fg_colors.push(col); }
            } else {
                if !bg_colors.contains(&col) { bg_colors.push(col); }
            }
        }
    }
    if fg_colors.len() == 1 && bg_colors.len() == 1 && (fg_colors[0] != bg_colors[0]) {
        Some((fg_colors[0], bg_colors[0]))
    } else if fg_colors.len() < 50 && bg_colors.len() < 50 {
        // blend each, fg and bg
        let fg_color = fg_colors.iter().fold(fg_colors[0], |acc, &x| {
            let mut c = acc.clone();
            c.blend(&x);
            c
        });
        let bg_color = bg_colors.iter().fold(bg_colors[0], |acc, &x| {
            let mut c = acc.clone();
            c.blend(&x);
            c
        });
        Some((fg_color, bg_color))
    } else {
        None
    }
}