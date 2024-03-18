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