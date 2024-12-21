// expanding our palette to https://en.wikipedia.org/wiki/Block_Elements

// Any characters that could be created by inverting one of these was removed.
// That leaves 20 in total.

use crate::printer::block::masks::{SUBPIXEL64_COLUMNS, SUBPIXEL64_ROWS};

// U+2581	▁	Lower one eighth block
const LOWER_ONE_EIGHTH_BLOCK: LabeledMasker<HorizontalMasker> = LabeledMasker('▁', HorizontalMasker(2));
// U+2582	▂	Lower one quarter block
const LOWER_ONE_QUARTER_BLOCK: LabeledMasker<HorizontalMasker> = LabeledMasker('▂', HorizontalMasker(4));
// U+2583	▃	Lower three eighths block
const LOWER_THREE_EIGHTHS_BLOCK: LabeledMasker<HorizontalMasker> = LabeledMasker('▃', HorizontalMasker(6));
// U+2584	▄	Lower half block
const LOWER_HALF_BLOCK: LabeledMasker<HorizontalMasker> = LabeledMasker('▄', HorizontalMasker(8));
// U+2585	▅	Lower five eighths block
const LOWER_FIVE_EIGHTHS_BLOCK: LabeledMasker<HorizontalMasker> = LabeledMasker('▅', HorizontalMasker(10));
// U+2586	▆	Lower three quarters block
const LOWER_THREE_QUARTERS_BLOCK: LabeledMasker<HorizontalMasker> = LabeledMasker('▆', HorizontalMasker(12));
// U+2587	▇	Lower seven eighths block
const LOWER_SEVEN_EIGHTHS_BLOCK: LabeledMasker<HorizontalMasker> = LabeledMasker('▇', HorizontalMasker(14));
// U+2588	█	Full block
const FULL_BLOCK: LabeledMasker<FullMasker> = LabeledMasker('█', FullMasker);
// U+2589	▉	Left seven eighths block
const LEFT_SEVEN_EIGHTHS_BLOCK: LabeledMasker<VerticalMasker> = LabeledMasker('▉', VerticalMasker(7));
// U+258A	▊	Left three quarters block
const LEFT_THREE_QUARTERS_BLOCK: LabeledMasker<VerticalMasker> = LabeledMasker('▊', VerticalMasker(6));
// U+258B	▋	Left five eighths block
const LEFT_FIVE_EIGHTHS_BLOCK: LabeledMasker<VerticalMasker> = LabeledMasker('▋', VerticalMasker(5));
// U+258C	▌	Left half block
const LEFT_HALF_BLOCK: LabeledMasker<VerticalMasker> = LabeledMasker('▌', VerticalMasker(4));
// U+258D	▍	Left three eighths block
const LEFT_THREE_EIGHTHS_BLOCK: LabeledMasker<VerticalMasker> = LabeledMasker('▍', VerticalMasker(3));
// U+258E	▎	Left one quarter block
const LEFT_ONE_QUARTER_BLOCK: LabeledMasker<VerticalMasker> = LabeledMasker('▎', VerticalMasker(2));
// U+258F	▏	Left one eighth block
const LEFT_ONE_EIGHTH_BLOCK: LabeledMasker<VerticalMasker> = LabeledMasker('▏', VerticalMasker(1));
// U+2596	▖	Quadrant lower left
const QUADRANT_LOWER_LEFT: LabeledMasker<AndMasker<VerticalMasker, HorizontalMasker>> = LabeledMasker('▖', AndMasker(VerticalMasker(4), HorizontalMasker(8)));
// U+2597	▗	Quadrant lower right
const QUADRANT_LOWER_RIGHT: LabeledMasker<AndMasker<InvertMasker<VerticalMasker>, HorizontalMasker>> = LabeledMasker('▗', AndMasker(InvertMasker(VerticalMasker(4)), HorizontalMasker(8)));
// U+2598	▘	Quadrant upper left
const QUADRANT_UPPER_LEFT: LabeledMasker<AndMasker<VerticalMasker, InvertMasker<HorizontalMasker>>> = LabeledMasker('▘', AndMasker(VerticalMasker(4), InvertMasker(HorizontalMasker(8))));
// U+259A	▚	Quadrant upper left and lower right
const QUADRANT_UPPER_LEFT_AND_LOWER_RIGHT: LabeledMasker<XorMasker<HorizontalMasker, VerticalMasker>> = LabeledMasker('▚', XorMasker(HorizontalMasker(4), VerticalMasker(8)));
// U+259D	▝	Quadrant upper right
const QUADRANT_UPPER_RIGHT: LabeledMasker<AndMasker<InvertMasker<VerticalMasker>, InvertMasker<HorizontalMasker>>> = LabeledMasker('▝', AndMasker(InvertMasker(VerticalMasker(4)), InvertMasker(HorizontalMasker(8))));

// ◢◣
const RAMP_UP: LabeledMasker<Linear> = LabeledMasker('◢', Linear(2., 0.));
const RAMP_DOWN: LabeledMasker<Linear> = LabeledMasker('◣', Linear(1., -4.));

const RAMP_1: LabeledMasker<Linear> = LabeledMasker('🭇', Linear(1., -4.));
const RAMP_2: LabeledMasker<Linear> = LabeledMasker('🭈', Linear(0.6, 0.));
const RAMP_3: LabeledMasker<Linear> = LabeledMasker('🭉', Linear(2., -8.));
const RAMP_4: LabeledMasker<Linear> = LabeledMasker('🭊', Linear(1., 0.));
const RAMP_5: LabeledMasker<Linear> = LabeledMasker('🭋', Linear(4., -16.));
const RAMP_6: LabeledMasker<Linear> = LabeledMasker('🭆', Linear(1., 4.));

/*
'🭇' => RAMP_1.1.mask(row, column),
'🭈' => RAMP_2.1.mask(row, column),
'🭉' => RAMP_3.1.mask(row, column),
'🭊' => RAMP_4.1.mask(row, column),
'🭋' => RAMP_5.1.mask(row, column),
'🭆' => RAMP_6.1.mask(row, column),
// 🭇 🭈 🭉 🭊 🭋 🭆
'🭑', RAMP_6.1.mask(row, SUBPIXEL64_COLUMNS - column),
'🭀', RAMP_5.1.mask(row, SUBPIXEL64_COLUMNS - column),
'🬿', RAMP_4.1.mask(row, SUBPIXEL64_COLUMNS - column),
'🬾', RAMP_3.1.mask(row, SUBPIXEL64_COLUMNS - column),
'🬽', RAMP_2.1.mask(row, SUBPIXEL64_COLUMNS - column),
'🬼', RAMP_1.1.mask(row, SUBPIXEL64_COLUMNS - column),
'🭢', RAMP_1.1.mask(SUBPIXEL64_ROWS - row, column),
'🭣', RAMP_2.1.mask(SUBPIXEL64_ROWS - row, column),
'🭤', RAMP_3.1.mask(SUBPIXEL64_ROWS - row, column),
'🭥', RAMP_4.1.mask(SUBPIXEL64_ROWS - row, column),
'🭦', RAMP_5.1.mask(SUBPIXEL64_ROWS - row, column),
'🭧', RAMP_6.1.mask(SUBPIXEL64_ROWS - row, column),
'🭜', RAMP_6.1.mask(SUBPIXEL64_ROWS - row, SUBPIXEL64_COLUMNS - column),
'🭛', RAMP_5.1.mask(SUBPIXEL64_ROWS - row, SUBPIXEL64_COLUMNS - column),
'🭚', RAMP_4.1.mask(SUBPIXEL64_ROWS - row, SUBPIXEL64_COLUMNS - column),
'🭙', RAMP_3.1.mask(SUBPIXEL64_ROWS - row, SUBPIXEL64_COLUMNS - column),
'🭘', RAMP_2.1.mask(SUBPIXEL64_ROWS - row, SUBPIXEL64_COLUMNS - column),
'🭗', RAMP_1.1.mask(SUBPIXEL64_ROWS - row, SUBPIXEL64_COLUMNS - column),
 */

// Now, instead of considering a 1x2 block at a time, we actually need to consider
// a 8x16 block.
// If 1 pixel is an 8x8, then we have 64 subpixels.

// Consider that the pixels are arranged in a row-major format, starting with 0,0

pub const ALL_BLOCK_ELEMENTS: &str = "▁▂▃▄▅▆▇█▉▊▋▌▍▎▏▖▗▘▚▝";

pub trait Masker {
    fn mask(&self, row: usize, column: usize) -> bool;
}

pub struct CharMasker(pub char);

struct LabeledMasker<M: ?Sized + Masker>(pub char, pub(crate) M);

struct FullMasker;

struct VerticalMasker(usize); // number of columns in the left
struct HorizontalMasker(usize); // number of rows on the bottom

struct InvertMasker<M: Masker>(M); // invert the masker
struct XorMasker<S: Masker, T: Masker>(S, T); // join 2 maskers together with NAND
struct AndMasker<S: Masker, T: Masker>(S, T); // join 2 maskers together with AND
struct OrMasker<S: Masker, T: Masker>(S, T); // join 2 maskers together, keeping all parts

struct Linear(f32, f32); // (m, b) => true if y >= mx + b

impl Masker for CharMasker {
    fn mask(&self, row: usize, column: usize) -> bool {
        match self.0 {
            '▁' => LOWER_ONE_EIGHTH_BLOCK.1.mask(row, column),
            '▂' => LOWER_ONE_QUARTER_BLOCK.1.mask(row, column),
            '▃' => LOWER_THREE_EIGHTHS_BLOCK.1.mask(row, column),
            '▄' => LOWER_HALF_BLOCK.1.mask(row, column),
            '▅' => LOWER_FIVE_EIGHTHS_BLOCK.1.mask(row, column),
            '▆' => LOWER_THREE_QUARTERS_BLOCK.1.mask(row, column),
            '▇' => LOWER_SEVEN_EIGHTHS_BLOCK.1.mask(row, column),
            '█' => FULL_BLOCK.1.mask(row, column),
            '▉' => LEFT_SEVEN_EIGHTHS_BLOCK.1.mask(row, column),
            '▊' => LEFT_THREE_QUARTERS_BLOCK.1.mask(row, column),
            '▋' => LEFT_FIVE_EIGHTHS_BLOCK.1.mask(row, column),
            '▌' => LEFT_HALF_BLOCK.1.mask(row, column),
            '▍' => LEFT_THREE_EIGHTHS_BLOCK.1.mask(row, column),
            '▎' => LEFT_ONE_QUARTER_BLOCK.1.mask(row, column),
            '▏' => LEFT_ONE_EIGHTH_BLOCK.1.mask(row, column),
            '▖' => QUADRANT_LOWER_LEFT.1.mask(row, column),
            '▗' => QUADRANT_LOWER_RIGHT.1.mask(row, column),
            '▘' => QUADRANT_UPPER_LEFT.1.mask(row, column),
            '▚' => QUADRANT_UPPER_LEFT_AND_LOWER_RIGHT.1.mask(row, column),
            '▝' => QUADRANT_UPPER_RIGHT.1.mask(row, column),
            '🭇' => RAMP_1.1.mask(row, column),
            '🭈' => RAMP_2.1.mask(row, column),
            '🭉' => RAMP_3.1.mask(row, column),
            '🭊' => RAMP_4.1.mask(row, column),
            '🭋' => RAMP_5.1.mask(row, column),
            '🭆' => RAMP_6.1.mask(row, column),
            '◢' => RAMP_UP.1.mask(row, column),
            '◣' => RAMP_DOWN.1.mask(row, column),
            '🭑' => RAMP_6.1.mask(row, SUBPIXEL64_COLUMNS as usize - column),
            '🭀' => RAMP_5.1.mask(row, SUBPIXEL64_COLUMNS as usize - column),
            '🬿' => RAMP_4.1.mask(row, SUBPIXEL64_COLUMNS as usize - column),
            '🬾' => RAMP_3.1.mask(row, SUBPIXEL64_COLUMNS as usize - column),
            '🬽' => RAMP_2.1.mask(row, SUBPIXEL64_COLUMNS as usize - column),
            '🬼' => RAMP_1.1.mask(row, SUBPIXEL64_COLUMNS as usize - column),
            '🭢' => RAMP_1.1.mask(SUBPIXEL64_ROWS as usize - row, column),
            '🭣' => RAMP_2.1.mask(SUBPIXEL64_ROWS as usize - row, column),
            '🭤' => RAMP_3.1.mask(SUBPIXEL64_ROWS as usize - row, column),
            '🭥' => RAMP_4.1.mask(SUBPIXEL64_ROWS as usize - row, column),
            '🭦' => RAMP_5.1.mask(SUBPIXEL64_ROWS as usize - row, column),
            '🭧' => RAMP_6.1.mask(SUBPIXEL64_ROWS as usize - row, column),
            '🭜' => RAMP_6.1.mask(SUBPIXEL64_ROWS as usize - row, SUBPIXEL64_COLUMNS as usize - column),
            '🭛' => RAMP_5.1.mask(SUBPIXEL64_ROWS as usize - row, SUBPIXEL64_COLUMNS as usize - column),
            '🭚' => RAMP_4.1.mask(SUBPIXEL64_ROWS as usize - row, SUBPIXEL64_COLUMNS as usize - column),
            '🭙' => RAMP_3.1.mask(SUBPIXEL64_ROWS as usize - row, SUBPIXEL64_COLUMNS as usize - column),
            '🭘' => RAMP_2.1.mask(SUBPIXEL64_ROWS as usize - row, SUBPIXEL64_COLUMNS as usize - column),
            '🭗' => RAMP_1.1.mask(SUBPIXEL64_ROWS as usize - row, SUBPIXEL64_COLUMNS as usize - column),
            _ => panic!("Unknown character '{}'", self.0)
        }
    }
}

impl<S: Masker, T: Masker> Masker for XorMasker<S, T> {
    fn mask(&self, row: usize, column: usize) -> bool {
        self.0.mask(row, column) ^ self.1.mask(row, column)
    }
}
impl<S: Masker, T: Masker> Masker for AndMasker<S, T> {
    fn mask(&self, row: usize, column: usize) -> bool {
        self.0.mask(row, column) && self.1.mask(row, column)
    }
}

impl<S: Masker, T: Masker> Masker for OrMasker<S, T> {
    fn mask(&self, row: usize, column: usize) -> bool {
        self.0.mask(row, column) || self.1.mask(row, column)
    }
}

impl Masker for VerticalMasker {
    fn mask(&self, row: usize, column: usize) -> bool {
        column < self.0
    }
}

impl Masker for HorizontalMasker {
    fn mask(&self, row: usize, column: usize) -> bool {
        (SUBPIXEL64_ROWS as usize - row - 1) < self.0
    }
}

impl Masker for Linear {
    fn mask(&self, row: usize, column: usize) -> bool {
        ((SUBPIXEL64_ROWS as usize - row) as f32) <= (column as f32 * self.0 + self.1)
    }
}

impl<M: Masker> Masker for InvertMasker<M> {
    fn mask(&self, row: usize, column: usize) -> bool {
        !self.0.mask(row, column)
    }
}

impl Masker for FullMasker {
    fn mask(&self, _row: usize, _column: usize) -> bool {
        true
    }
}