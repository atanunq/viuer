use image::{GenericImage, ImageBuffer, Pixel, Rgba, RgbaImage, SubImage};
use image::ColorType::Rgba8;
use termcolor::Color;
use crate::Config;
use crate::printer::block::{CHECKERBOARD_BACKGROUND_DARK, CHECKERBOARD_BACKGROUND_LIGHT};
use crate::printer::block::maskers::{CharMasker, Masker};

pub const SUBPIXEL64_ROWS: u32 = 16;
pub const SUBPIXEL64_COLUMNS: u32 = 8;

pub struct Mask {
    pub char: char,
    pub mask: [[bool; SUBPIXEL64_COLUMNS as usize]; SUBPIXEL64_ROWS as usize]
}

impl Mask {
    pub fn new(masker: CharMasker) -> Mask {
        let mut mask = [[false; SUBPIXEL64_COLUMNS as usize]; SUBPIXEL64_ROWS as usize];
        for row in 0..SUBPIXEL64_ROWS as usize {
            for column in 0..SUBPIXEL64_COLUMNS as usize {
                let m = masker.mask(row, column);
                mask[row][column] = m;
            }
        }
        Mask { char: masker.0, mask }
    }
}

pub fn get_mask_for_char(mask_char: char) -> Mask {
    Mask::new(CharMasker(mask_char))
}

pub fn get_all_masks() -> Vec<Mask> {
    // ALL_BLOCK_ELEMENTS
    //    "▁▂▃▄▅▆▇▉▊▋▌▍▎▏▖▗▘▚▝" "◢◣"
       "▁▂▃▄▅▆▇▉▊▋▌▍▎▏▖▗▘▚▝🭇🭈🭉🭊🭋🭆🭑🭀🬿🬾🬽🬼🭢🭣🭤🭥🭦🭧🭜🭛🭚🭙🭘🭗"
    // "▁▂▃▄▅▆▇█"
        .chars()
        .map(|c| CharMasker(c))
        .map(|cm| Mask::new(cm))
        .collect()
}
pub fn choose_mask_and_colors<'a>(masks: &'a Vec<Mask>, offset: (u32, u32), img: &RgbaImage, config: &Config) -> Option<(&'a Mask, Option<Color>, Option<Color>)> {
    let mut selected = None;
    let mut selected_score = -1.0;
    for mask in masks.iter() {
        if let Some((score, Some(c1), Some(c2))) = get_mask_colors(img, config, offset, &mask) {
            assert!(score >= 0.);
            // println!("Score for mask {}: {}", &mask.char, score);
            if selected_score < 0. || score < selected_score {
                selected = Some((mask, Some(c1), Some(c2)));
                selected_score = score;
            }
        }
    }
    selected
}

fn as_f64_col(col: Rgba<u8>) -> Rgba<f64> {
    Rgba([col[0] as f64, col[1] as f64, col[2] as f64, col[3] as f64])
}

/// Returns an error amount, and the fg and bg colors
/// error >= 0
pub fn get_mask_colors(img: &RgbaImage, config: &Config, (x_offset, y_offset): (u32, u32), mask: &Mask) -> Option<(f64, Option<Color>, Option<Color>)> {
    let mut fg_colors = vec![];
    let mut bg_colors = vec![];
    let (width, height) = img.dimensions();
    for row in 0..SUBPIXEL64_ROWS {
        for column in 0..SUBPIXEL64_COLUMNS {
            let x = column + x_offset;
            let y = row + y_offset;
            if x >= width { continue }
            if y >= height { continue }
            let mut col = *img.get_pixel(x, y);
            if col[3] == 0 && !config.transparent {
                let t = if (x/SUBPIXEL64_COLUMNS) % 2 == (y/SUBPIXEL64_COLUMNS) % 2 {
                    CHECKERBOARD_BACKGROUND_DARK
                } else { CHECKERBOARD_BACKGROUND_LIGHT };
                col = Rgba([t.0, t.1, t.2, 255]);
            }
            if mask.mask[row as usize][column as usize] {
                fg_colors.push(col);
            } else {
                bg_colors.push(col);
            }
        }
    }
    let error = color_stdev(&fg_colors) + color_stdev(&bg_colors);
    let fg_avg = color_avg(&fg_colors);
    let bg_avg = color_avg(&bg_colors);
    let map_a = |c: Rgba<u8>| {
        if c[3] < 128 {
            // Some(Color::Red)
            None
        } else {
            Some(Color::Rgb(c[0], c[1], c[2]))
        }
    };
    Some((error, Some(fg_avg).and_then(map_a), Some(bg_avg).and_then(map_a)))
}

fn color_dist(c1: &Rgba<u8>, c2: &Rgba<u8>) -> f64 {
    let r = c1[0] as f64 - c2[0] as f64;
    let g = c1[1] as f64 - c2[1] as f64;
    let b = c1[2] as f64 - c2[2] as f64;
    let a = c1[3] as f64 - c2[3] as f64;
    let aa = std::cmp::min(c1[3], c2[3]) as f64 / 2. / 255.;
    (r * r / 255. + g * g / 255. + b * b / 255.) * aa + a * a * 3.
}

fn color_avg(colors: &Vec<Rgba<u8>>) -> Rgba<u8> {
    // if colors.len() == 0 { return Rgba([0, 0, 0, 255]); }
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    let mut a = 0;
    for c in colors {
        r += c[0] as u32;
        g += c[1] as u32;
        b += c[2] as u32;
        a += c[3] as u32;
    }
    let len = colors.len() as u32;
    Rgba([(r / len) as u8, (g / len) as u8, (b / len) as u8, (a / len) as u8])
}

fn color_median(colors: &Vec<Rgba<u8>>) -> Rgba<u8> {
    if colors.len() == 0 {
        return Rgba([0, 0, 0, 0]);
    }
    // get average
    let avg_color = color_avg(colors);
    // sort them by distance to average
    let mut sorted = colors.clone();
    sorted.sort_by(|a, b| color_dist(a, &avg_color).partial_cmp(&color_dist(b, &avg_color)).unwrap());
    // pick closest one
    sorted[0]
}

fn color_stdev_with_avg(colors: &Vec<Rgba<u8>>, avg: &Rgba<u8>) -> f64 {
    colors.iter().map(|c| color_dist(c, avg)).sum::<f64>() / colors.len() as f64
}
fn color_stdev(colors: &Vec<Rgba<u8>>) -> f64 {
    if colors.len() == 0 { return 0.; }
    let avg = color_avg(colors);
    let stdev = color_stdev_with_avg(colors, &avg);
    stdev * stdev
}

fn colors_by_count(colors: &Vec<Rgba<u8>>) -> Vec<Rgba<u8>> {
    // sort the colors by the number of times they occur
    let mut counts = std::collections::HashMap::new();
    for c in colors {
        *counts.entry(c).or_insert(0) += 1;
    }
    let mut new_colors = colors.clone();
    new_colors.sort_by_key(|c| -counts[c]);
    new_colors
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use image::{DynamicImage, Rgba, RgbaImage};
    use termcolor::{BufferedStandardStream, ColorChoice};
    use crate::Config;
    use crate::printer::{BlockPrinter, Printer};
    use crate::printer::block::masks::{choose_mask_and_colors, get_all_masks, get_mask_colors, get_mask_for_char};

    fn print_color(color: Rgba<u8>) {
        // create a dynamic image
        let img = RgbaImage::from_pixel(1, 1, color);
        let img = DynamicImage::ImageRgba8(img);
        let mut stream = BufferedStandardStream::stdout(ColorChoice::Always);
        let conf = Config {
            width: Some(1),
            height: Some(1),
            ..Config::default()
        };
        BlockPrinter.print(&mut stream, &img, &conf).expect("Image printing failed.");
    }

    #[test]
    fn sub_img_tests() {
        let make_conf = |width, height| Config {
            x: 0, y: 0,
            width: Some(width), height: Some(height),
            ..Default::default()
        };
        let dir = Path::new("/home/veggiebob/Documents/viuer-test-assets/");
        let file_prefix = "viuer-test-assets_";
        let file_suffix = ".png";

        let filename = "0001";

        let filename = format!("{}{}{}", file_prefix, filename, file_suffix);
        let filepath = dir.join(filename);

        let img = image::io::Reader::open(&filepath).unwrap()
            .with_guessed_format().unwrap().decode().unwrap();
        let mut stream = BufferedStandardStream::stdout(ColorChoice::Always);
        // BlockPrinter.print(&mut stream, &img, &make_conf(8, 16)).unwrap();
        let rgbimg = img.to_rgba8();
        let mask = get_mask_for_char('▄');
        let config = make_conf(8, 16);
        let (score, fg, bg) = get_mask_colors(&rgbimg, &config,(0, 0), &mask).unwrap();
        println!("Score: {}", score);
        println!("Colors: {:?} and {:?}", fg, bg);
        // print_color(fg);
        // print_color(bg);
    }

    #[test]
    fn choose_mask_test() {
        let make_conf = |width, height| Config {
            x: 0, y: 0,
            width: Some(width), height: Some(height),
            ..Default::default()
        };
        let dir = Path::new("/home/veggiebob/Documents/viuer-test-assets/");
        let file_prefix = "viuer-test-assets_";
        let file_suffix = ".png";

        let filename = "0001";

        let filename = format!("{}{}{}", file_prefix, filename, file_suffix);
        let filepath = dir.join(filename);

        let img = image::io::Reader::open(&filepath).unwrap()
            .with_guessed_format().unwrap().decode().unwrap();
        let mut stream = BufferedStandardStream::stdout(ColorChoice::Always);
        BlockPrinter.print(&mut stream, &img, &make_conf(8, 16)).unwrap();
        let rgbimg = img.to_rgba8();
        let config = make_conf(8, 16);
        let mask_cache = get_all_masks();
        let (mask, fg, bg) = choose_mask_and_colors(&mask_cache, (0, 0), &rgbimg, &config).unwrap();
        println!("Mask: {}", mask.char);
        println!("Colors: {:?} and {:?}", fg, bg);
        // print_color(fg);
        // print_color(bg);
    }
}