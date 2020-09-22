use crate::error::{ViuError, ViuResult};
use crate::printer::Printer;
use crate::Config;

use ansi_colours::ansi256_from_rgb;
use image::{DynamicImage, GenericImageView, Rgba};
use std::io::Write;
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

const UPPER_HALF_BLOCK: &str = "\u{2580}";
const LOWER_HALF_BLOCK: &str = "\u{2584}";
const EMPTY_BLOCK: &str = " ";

pub struct BlockPrinter {}

impl Printer for BlockPrinter {
    fn print(img: &DynamicImage, config: &Config) -> ViuResult {
        // there are two types of buffers in this function:
        // - stdout: Buffer, which is from termcolor crate. Used to buffer all writing
        //   required to print a single image or frame. Flushed once at the end of the function
        // - buffer: Vec<ColorSpec>, which stores back- and foreground colors for a
        //   maximum of 1 row of blocks, i.e 2 rows of pixels. Flushed every 2 pixel rows of the images
        // all mentions of buffer below refer to the latter
        let out = BufferWriter::stdout(ColorChoice::Always);
        let mut stdout = out.buffer();

        let (width, _) = img.dimensions();

        let mut curr_col_px = 0;
        let mut curr_row_px = 0;
        let mut buffer: Vec<ColorSpec> = Vec::with_capacity(width as usize);
        let mut mode = Mode::Top;

        // iterate pixels and fill a buffer that contains 2 rows of pixels
        // 2 rows translate to 1 row in the terminal by using half block, foreground and background
        // colors
        for pixel in img.pixels() {
            // if the alpha of the pixel is 0, print a predefined pixel based on the position in order
            // to mimic the chess board background. If the transparent option was given, instead print
            // nothing.
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
                let mut c = ColorSpec::new();
                c.set_bg(color);
                buffer.push(c);
            } else {
                let colorspec_to_upg = &mut buffer[curr_col_px as usize];
                colorspec_to_upg.set_fg(color);
            }

            curr_col_px += 1;
            // if the buffer is full start adding the second row of pixels
            if buffer.len() == width as usize {
                if mode == Mode::Top {
                    mode = Mode::Bottom;
                    curr_col_px = 0;
                    curr_row_px += 1;
                }
                // only if the second row is completed flush the buffer and start again
                else if curr_col_px == width {
                    curr_col_px = 0;
                    curr_row_px += 1;
                    print_buffer(&mut buffer, false, &mut stdout)?;
                    mode = Mode::Top;
                } else {
                    // we are in the middle of the second row, there is work to do
                }
            }
        }

        // buffer will be flushed if the image has an odd height
        if !buffer.is_empty() {
            print_buffer(&mut buffer, true, &mut stdout)?;
        }

        match out.print(&stdout) {
            Ok(_) => Ok(()),
            Err(e) => match e.kind() {
                // Ignore broken pipe errors. They arise when piping output to `head`, for example,
                // and panic is not desired.
                std::io::ErrorKind::BrokenPipe => Ok(()),
                _ => Err(ViuError::IO(e)),
            },
        }
    }
}

fn print_buffer(buff: &mut Vec<ColorSpec>, is_flush: bool, stdout: &mut Buffer) -> ViuResult {
    let mut out_color;
    let mut out_char;
    let mut new_color;

    for c in buff.iter() {
        // If a flush is needed it means that only one row with UPPER_HALF_BLOCK must be printed
        // because it is the last row, hence it contains only 1 pixel
        if is_flush {
            new_color = ColorSpec::new();
            if let Some(bg) = c.bg() {
                new_color.set_fg(Some(*bg));
                out_char = UPPER_HALF_BLOCK;
            } else {
                out_char = EMPTY_BLOCK;
            }
            out_color = &new_color;
        } else {
            match (c.fg(), c.bg()) {
                (None, None) => {
                    // completely transparent
                    new_color = ColorSpec::new();
                    out_color = &new_color;
                    out_char = EMPTY_BLOCK;
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
        change_stdout_color(stdout, out_color)?;
        write!(stdout, "{}", out_char)?;
    }

    clear_printer(stdout)?;
    write_newline(stdout)?;
    buff.clear();

    Ok(())
}

fn is_pixel_transparent(pixel: (u32, u32, Rgba<u8>)) -> bool {
    let (_x, _y, data) = pixel;
    data[3] == 0
}

fn get_transparency_color(row: u32, col: u32, truecolor: bool) -> Color {
    //imitate the transparent chess board pattern
    let rgb = if row % 2 == col % 2 {
        (102, 102, 102)
    } else {
        (153, 153, 153)
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

fn clear_printer(stdout: &mut Buffer) -> ViuResult {
    let c = ColorSpec::new();
    change_stdout_color(stdout, &c)
}

fn change_stdout_color(stdout: &mut Buffer, color: &ColorSpec) -> ViuResult {
    stdout.set_color(color).map_err(ViuError::IO)
}

fn write_newline(stdout: &mut Buffer) -> ViuResult {
    writeln!(stdout).map_err(ViuError::IO)
}

// enum used to keep track where the current line of pixels processed should be displayed - as
// background or foreground color
#[derive(PartialEq)]
enum Mode {
    Top,
    Bottom,
}
