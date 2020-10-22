use crate::error::{ViuError, ViuResult};
use crate::printer::Printer;
use crate::Config;

use ansi_colours::ansi256_from_rgb;
use image::{DynamicImage, GenericImageView, Rgba};
use std::io::Write;
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

use crossterm::cursor::{MoveRight, MoveTo, MoveToPreviousLine};
use crossterm::execute;

const UPPER_HALF_BLOCK: &str = "\u{2580}";
const LOWER_HALF_BLOCK: &str = "\u{2584}";

const CHECKERBOARD_BACKGROUND_LIGHT: (u8, u8, u8) = (153, 153, 153);
const CHECKERBOARD_BACKGROUND_DARK: (u8, u8, u8) = (102, 102, 102);

pub struct BlockPrinter {}

impl Printer for BlockPrinter {
    fn print(img: &DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
        // there are two types of buffers in this function:
        // - stdout: Buffer, which is from termcolor crate. Used to buffer all writing
        //   required to print a single image or frame. Flushed on every line
        // - row_buffer: Vec<ColorSpec>, which stores back- and foreground colors for a
        //   row of terminal cells. When flushed, its output goes into out_buffer.
        // They are both flushed on every terminal line (i.e 2 pixel rows)
        let stdout = BufferWriter::stdout(ColorChoice::Always);
        let mut out_buffer = stdout.buffer();

        // adjust y offset
        if config.absolute_offset {
            if config.y >= 0 {
                // If absolute_offset, move to (0,y).
                execute!(out_buffer, MoveTo(0, config.y as u16))?;
            } else {
                //Negative values do not make sense.
                return Err(ViuError::InvalidConfiguration(
                    "absolute_offset is true but y offset is negative".to_owned(),
                ));
            }
        } else if config.y < 0 {
            // MoveUp if negative
            execute!(out_buffer, MoveToPreviousLine(-config.y as u16))?;
        } else {
            // Move down y lines
            for _ in 0..config.y {
                // writeln! is used instead of MoveDown to force scrolldown
                // observed when config.y > 0 and cursor is on the last terminal line
                writeln!(out_buffer)?;
            }
        }

        // resize the image so that it fits in the constraints, if any
        let resized_img;
        let img = if config.resize {
            resized_img = super::resize(&img, config.width, config.height);
            &resized_img
        } else {
            img
        };

        let (width, _) = img.dimensions();

        //TODO: position information is contained in the pixel
        let mut curr_col_px = 0;
        let mut curr_row_px = 0;

        let mut row_buffer: Vec<ColorSpec> = Vec::with_capacity(width as usize);

        // row_buffer building mode. At first the top colors are calculated and then the bottom
        // Once the bottom row is ready, row_buffer is flushed
        let mut mode = Mode::Top;

        // iterate pixels and fill row_buffer
        for pixel in img.pixels() {
            // if the alpha of the pixel is 0, print a predefined pixel based on the position in order
            // to mimic the checherboard background. If the transparent option was given, move right instead
            let color = if is_pixel_transparent(pixel) {
                if config.transparent {
                    None
                } else {
                    Some(get_transparency_color(
                        curr_row_px,
                        curr_col_px,
                        config.truecolor,
                    ))
                }
            } else {
                Some(get_color_from_pixel(pixel, config.truecolor))
            };

            if mode == Mode::Top {
                // add a new ColorSpec to row_buffer
                let mut c = ColorSpec::new();
                c.set_bg(color);
                row_buffer.push(c);
            } else {
                // upgrade an already existing ColorSpec
                let colorspec_to_upg = &mut row_buffer[curr_col_px as usize];
                colorspec_to_upg.set_fg(color);
            }

            curr_col_px += 1;
            // if the buffer is full start adding the second row of pixels
            if row_buffer.len() == width as usize {
                if mode == Mode::Top {
                    mode = Mode::Bottom;
                    curr_col_px = 0;
                    curr_row_px += 1;
                }
                // only if the second row is completed, flush the buffer and start again
                else if curr_col_px == width {
                    curr_col_px = 0;
                    curr_row_px += 1;

                    // move right if x offset is specified
                    if config.x > 0 {
                        execute!(out_buffer, MoveRight(config.x))?;
                    }

                    // flush the row_buffer into out_buffer
                    fill_out_buffer(&mut row_buffer, &mut out_buffer, false)?;

                    // write the line to stdout
                    print_buffer(&stdout, &mut out_buffer)?;

                    mode = Mode::Top;
                } else {
                    // in the middle of the second row, more iterations are required
                }
            }
        }

        // buffer will be flushed if the image has an odd height
        if !row_buffer.is_empty() {
            fill_out_buffer(&mut row_buffer, &mut out_buffer, true)?;
        }

        // do a final write to stdout to print last row if length is odd, and reset cursor position
        print_buffer(&stdout, &mut out_buffer)?;

        //TODO: might be +1/2 ?
        Ok((width, curr_row_px / 2))
    }
}

// Send out_buffer to stdout. Empties it when it's done
fn print_buffer(stdout: &BufferWriter, out_buffer: &mut Buffer) -> ViuResult {
    match stdout.print(out_buffer) {
        Ok(_) => {
            out_buffer.clear();
            Ok(())
        }
        Err(e) => match e.kind() {
            // Ignore broken pipe errors. They arise when piping output to `head`, for example,
            // and panic is not desired.
            std::io::ErrorKind::BrokenPipe => Ok(()),
            _ => Err(ViuError::IO(e)),
        },
    }
}

// Translates the row_buffer, containing colors, into the out_buffer which will be flushed to the terminal
fn fill_out_buffer(
    row_buffer: &mut Vec<ColorSpec>,
    out_buffer: &mut Buffer,
    is_last_row: bool,
) -> ViuResult {
    let mut out_color;
    let mut out_char;
    let mut new_color;

    for c in row_buffer.iter() {
        // If a flush is needed it means that only one row with UPPER_HALF_BLOCK must be printed
        // because it is the last row, hence it contains only 1 pixel
        if is_last_row {
            new_color = ColorSpec::new();
            if let Some(bg) = c.bg() {
                new_color.set_fg(Some(*bg));
                out_char = UPPER_HALF_BLOCK;
            } else {
                execute!(out_buffer, MoveRight(1))?;
                continue;
            }
            out_color = &new_color;
        } else {
            match (c.fg(), c.bg()) {
                (None, None) => {
                    // completely transparent
                    execute!(out_buffer, MoveRight(1))?;
                    continue;
                }
                (Some(bottom), None) => {
                    // only top transparent
                    new_color = ColorSpec::new();
                    new_color.set_fg(Some(*bottom));
                    out_color = &new_color;
                    out_char = LOWER_HALF_BLOCK;
                }
                (None, Some(top)) => {
                    // only bottom transparent
                    new_color = ColorSpec::new();
                    new_color.set_fg(Some(*top));
                    out_color = &new_color;
                    out_char = UPPER_HALF_BLOCK;
                }
                (Some(_top), Some(_bottom)) => {
                    // both parts have a color
                    out_color = c;
                    out_char = LOWER_HALF_BLOCK;
                }
            }
        }
        out_buffer.set_color(out_color)?;
        write!(out_buffer, "{}", out_char)?;
    }

    out_buffer.reset()?;
    writeln!(out_buffer)?;
    row_buffer.clear();

    Ok(())
}

fn is_pixel_transparent(pixel: (u32, u32, Rgba<u8>)) -> bool {
    let (_x, _y, data) = pixel;
    data[3] == 0
}

fn get_transparency_color(row: u32, col: u32, truecolor: bool) -> Color {
    //imitate the transparent chess board pattern
    let rgb = if row % 2 == col % 2 {
        CHECKERBOARD_BACKGROUND_DARK
    } else {
        CHECKERBOARD_BACKGROUND_LIGHT
    };
    if truecolor {
        Color::Rgb(rgb.0, rgb.1, rgb.2)
    } else {
        Color::Ansi256(ansi256_from_rgb(rgb))
    }
}

fn get_color_from_pixel(pixel: (u32, u32, Rgba<u8>), truecolor: bool) -> Color {
    let (_x, _y, data) = pixel;
    let rgb = (data[0], data[1], data[2]);
    if truecolor {
        Color::Rgb(rgb.0, rgb.1, rgb.2)
    } else {
        Color::Ansi256(ansi256_from_rgb(rgb))
    }
}

// enum used to keep track where the current line of pixels processed should be displayed - as
// background or foreground color
#[derive(PartialEq)]
enum Mode {
    Top,
    Bottom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_printer_small() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::new(20, 6));

        let config = Config {
            width: Some(40),
            height: None,
            absolute_offset: false,
            transparent: true,
            ..Default::default()
        };
        let (w, h) = BlockPrinter::print(&img, &config).unwrap();

        assert_eq!(w, 20);
        assert_eq!(h, 3);
    }
    #[test]
    fn test_block_printer_large() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::new(2000, 1000));

        let config = Config {
            width: Some(160),
            height: None,
            absolute_offset: false,
            transparent: true,
            ..Default::default()
        };
        let (w, h) = BlockPrinter::print(&img, &config).unwrap();

        assert_eq!(w, 160);
        assert_eq!(h, 40);
    }
}
