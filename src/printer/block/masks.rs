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
    // ALL_BLOCK_ELEMENTS
    // "▁▂▃▄▅▆▇█▉▊▋▌▍▎▏▖▗▘▚▝"
    "▁▂▃▄▅▆▇█▉▊▋▌▍▎▏▖▗▘▚▝"
    // "▁▂▃▄▅▆▇█"
        .chars()
        .map(|c| CharMasker(c))
        .map(|cm| Mask::new(cm))
        .collect()
}

pub fn get_mask_colors(img: &RgbaImage, mask: &Mask) -> Option<(f64, Rgba<u8>, Rgba<u8>)> {
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
                fg_colors.push(col);
            } else {
                bg_colors.push(col);
            }
        }
    }
    if fg_colors.len() == 1 && bg_colors.len() == 1 && (fg_colors[0] != bg_colors[0]) {
        Some((0.0, fg_colors[0], bg_colors[0]))
    } else if fg_colors.len() > 0 && bg_colors.len() > 0 {
        // println!("fg stdev: {}, bg stdev: {}", color_stdev(&fg_colors), color_stdev(&bg_colors));
        const STDEV_LIMIT: f64 = 80.0;
        let fg_stdev = color_stdev(&fg_colors);
        let bg_stdev = color_stdev(&bg_colors);
        if fg_stdev > STDEV_LIMIT || bg_stdev > STDEV_LIMIT {
            return None;
        }
        let fg_color = colors_by_count(&fg_colors)[0];
        let bg_color = colors_by_count(&bg_colors)[0];
        Some((1. / (fg_stdev * bg_stdev), fg_color, bg_color))
    } else {
        None
    }
}

fn color_dist(c1: &Rgba<u8>, c2: &Rgba<u8>) -> f64 {
    let r = c1[0] as f64 - c2[0] as f64;
    let g = c1[1] as f64 - c2[1] as f64;
    let b = c1[2] as f64 - c2[2] as f64;
    let a = c1[3] as f64 - c2[3] as f64;
    r * r + g * g + b * b + a * a
}

fn color_avg(colors: &Vec<Rgba<u8>>) -> Rgba<u8> {
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
    (colors.iter().map(|c| color_dist(c, avg)).sum::<f64>() / colors.len() as f64).sqrt()
}
fn color_stdev(colors: &Vec<Rgba<u8>>) -> f64 {
    let avg = color_avg(colors);
    color_stdev_with_avg(colors, &avg)
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

fn color_kmeans(colors: &Vec<Rgba<u8>>, means: u8, iterations: u8) -> Vec<Rgba<u8>> {
    let mut centroids = vec![];
    for i in 0..means {
        centroids.push(colors[i as usize]);
    }
    for _ in 0..iterations {
        let mut clusters = vec![vec![]; means as usize];
        for c in colors {
            let mut min_dist = std::f64::MAX;
            let mut min_idx = 0;
            for (i, cent) in centroids.iter().enumerate() {
                let dist = color_dist(c, cent);
                if dist < min_dist {
                    min_dist = dist;
                    min_idx = i;
                }
            }
            clusters[min_idx].push(*c);
        }
        for (i, cluster) in clusters.iter().enumerate() {
            if cluster.len() > 0 {
                centroids[i] = color_avg(cluster);
            }
        }
    }
    centroids
}