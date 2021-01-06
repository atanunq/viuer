Code Relay:


Task #15:

Found the origin of the escape code. It's an xterm specific settingsS
https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_

- Check the TERM variable to see if it is a supported term.
- If it is xterm. Check the device attributes

CSI Ps c  Send Device Attributes (Primary DA).
Ps = 0  or omitted ⇒  request attributes from terminal.  The
response depends on the decTerminalID resource setting.
  ⇒  CSI ? 1 ; 2 c  ("VT100 with Advanced Video Option")
  ⇒  CSI ? 1 ; 0 c  ("VT101 with No Options")
  ⇒  CSI ? 4 ; 6 c  ("VT132 with Advanced Video and Graphics")
  ⇒  CSI ? 6 c  ("VT102")
  ⇒  CSI ? 7 c  ("VT131")
  ⇒  CSI ? 1 2 ; Ps c  ("VT125")
  ⇒  CSI ? 6 2 ; Ps c  ("VT220")
  ⇒  CSI ? 6 3 ; Ps c  ("VT320")
  ⇒  CSI ? 6 4 ; Ps c  ("VT420")

The VT100-style response parameters do not mean anything by
themselves.  VT220 (and higher) parameters do, telling the
host what features the terminal supports:
  Ps = 1  ⇒  132-columns.
  Ps = 2  ⇒  Printer.
  Ps = 3  ⇒  ReGIS graphics.
  **Ps = 4  ⇒  Sixel graphics.**
  Ps = 6  ⇒  Selective erase.
  Ps = 8  ⇒  User-defined keys.
  Ps = 9  ⇒  National Replacement Character sets.
  Ps = 1 5  ⇒  Technical characters.
  Ps = 1 6  ⇒  Locator port.
  Ps = 1 7  ⇒  Terminal state interrogation.
  Ps = 1 8  ⇒  User windows.
  Ps = 2 1  ⇒  Horizontal scrolling.
  Ps = 2 2  ⇒  ANSI color, e.g., VT525.
  Ps = 2 8  ⇒  Rectangular editing.
  Ps = 2 9  ⇒  ANSI text locator (i.e., DEC Locator mode).


Task #14:
Complete sixel::check_sixel_support
Check for sixel compatibility in the main print method. If that is the case, call SixelPrinter, which lives in sixel.rs and implements the Printer trait.
- Look at the terminal requirements for libsixel https://github.com/saitoha/libsixel#terminal-requirements

- This comment on mintty claims "one can detect SIXEL capability using an escape sequence" https://github.com/mintty/mintty/issues/866#issuecomment-482781112. Find out what that escape sequence is
- Found it  https://github.com/gizak/termui/pull/233#issuecomment-478193544
the important ANSI escape strings here are:

"\033[0c" for querying the terminal capabilities - we need a 4 for sixel.

We also need to know what the size of a character box is in pixels this differs from terminal to terminal:

"\033[14t" gives us the terminal size in pixels

"\033[18t" gives us the terminal size in cells

from that we can calculate the cell size in pixels.

We set the position with "\033[%d;%dH" before printing the sixel string. the position string is simply prepended.

The first %d is the Y position (in cells) and the second the X position.\
- Find a way to read value in from terminal after running escape code to check if it is 4
- Find a way to test the escape code without writing the value to the current console.

Task #13:
Complete sixel::check_sixel_support
Check for sixel compatibility in the main print method. If that is the case, call SixelPrinter, which lives in sixel.rs and implements the Printer trait.
- Look at the terminal requirements for libsixel https://github.com/saitoha/libsixel#terminal-requirements

- This comment on mintty claims "one can detect SIXEL capability using an escape sequence" https://github.com/mintty/mintty/issues/866#issuecomment-482781112. Find out what that escape sequence is
- Found it  https://github.com/gizak/termui/pull/233#issuecomment-478193544
the important ANSI escape strings here are:

"\033[0c" for querying the terminal capabilities - we need a 4 for sixel.

We also need to know what the size of a character box is in pixels this differs from terminal to terminal:

"\033[14t" gives us the terminal size in pixels

"\033[18t" gives us the terminal size in cells

from that we can calculate the cell size in pixels.

We set the position with "\033[%d;%dH" before printing the sixel string. the position string is simply prepended.

The first %d is the Y position (in cells) and the second the X position.\
- Find a way to read value in from terminal after running escape code to check if it is 4


Task #12:
Complete sixel::check_sixel_support
Check for sixel compatibility in the main print method. If that is the case, call SixelPrinter, which lives in sixel.rs and implements the Printer trait.
- Look at the terminal requirements for libsixel https://github.com/saitoha/libsixel#terminal-requirements

- This comment on mintty claims "one can detect SIXEL capability using an escape sequence" https://github.com/mintty/mintty/issues/866#issuecomment-482781112. Find out what that escape sequence is
- Found it  https://github.com/gizak/termui/pull/233#issuecomment-478193544
the important ANSI escape strings here are:

"\033[0c" for querying the terminal capabilities - we need a 4 for sixel.

We also need to know what the size of a character box is in pixels this differs from terminal to terminal:

"\033[14t" gives us the terminal size in pixels

"\033[18t" gives us the terminal size in cells

from that we can calculate the cell size in pixels.

We set the position with "\033[%d;%dH" before printing the sixel string. the position string is simply prepended.

The first %d is the Y position (in cells) and the second the X position.


Task #11:
Complete sixel::check_sixel_support
Check for sixel compatibility in the main print method. If that is the case, call SixelPrinter, which lives in sixel.rs and implements the Printer trait.
- Look at the terminal requirements for libsixel https://github.com/saitoha/libsixel#terminal-requirements

- This comment on mintty claims "one can detect SIXEL capability using an escape sequence" https://github.com/mintty/mintty/issues/866#issuecomment-482781112. Find out what that escape sequence is

Task #10:
Complete sixel::check_sixel_support
Check for sixel compatibility in the main print method. If that is the case, call SixelPrinter, which lives in sixel.rs and implements the Printer trait.
- Look at the terminal requirements for libsixel https://github.com/saitoha/libsixel#terminal-requirements

Task #9:
Complete sixel::check_sixel_support
Check for sixel compatibility in the main print method. If that is the case, call SixelPrinter, which lives in sixel.rs and implements the Printer trait.

Task #8:

Add SixelError to impl std::fmt::Display for ViuError .


Task #7:
Remove MResult from printer sixel. Make it all ViuResult

Task #6:

Make SixlPrinter print return a ViuResult instead of an MResult

Task #5;

Make a SixlPrinter struct and implement printer using
sixel_print. (in printer/sixel)

Task #4:

- Get print_sixel to compile. Get type definitions from 
https://github.com/rabite0/hunter/blob/355d9a3101f6d8dc375807de79e368602f1cb87d/src/hunter-media.rs#L714-L745

Added print_sixel to printer::mod.rs

Task #3:
Task 2 was not completed as mentioned by author, so I actually added sixel to cargo.

Still need to complete sixel.rs, looking at example in issue. Right now sixel.rs is just a copy of kitty.rs

Task #2:
Implement this
https://github.com/atanunq/viuer/issues/4#issue-712252749
* already added sixel to cargo
* Complete sixel.rs, look at example in issue

---
Task #1:
Implement this
https://github.com/atanunq/viuer/issues/4#issue-712252749

---
# viuer
Display images in the terminal with ease.

![ci](https://github.com/atanunq/viuer/workflows/ci/badge.svg)

`viuer` is a Rust library that makes it easy to show images in the
terminal. It has a straightforward interface and is configured
through a single struct. The default printing method is through
lower half blocks (▄ or \u2585). However some custom graphics
protocols are supported. They result in full resolution images
being displayed in specific environments:

- [Kitty](https://sw.kovidgoyal.net/kitty/graphics-protocol.html)
- [iTerm](https://iterm2.com/documentation-images.html)

For a demo of the library's usage and example screenshots, see [`viu`](https://github.com/atanunq/viu).

## Examples

```toml
# in Cargo.toml, under [dependencies]
viuer = "0.3"
```
```rust
// in src/main.rs
use viuer::{print_from_file, Config};

fn main() {
    let conf = Config {
        // set offset
        x: 20,
        y: 4,
        // set dimensions
        width: Some(80),
        height: Some(25),
        ..Default::default()
    };

    // starting from row 4 and column 20,
    // display `img.jpg` with dimensions 80x25 (in terminal cells)
    // note that the actual resolution in the terminal will be 80x50
    print_from_file("img.jpg", &conf).expect("Image printing failed.");
}
```

Or if you have a [DynamicImage](https://docs.rs/image/*/image/enum.DynamicImage.html), you can use it directly:
```rust
// ..Config setup

let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(20, 10));
viuer::print(&img, &conf).expect("Image printing failed.");
```

## Docs
Check the [full documentation](https://docs.rs/crate/viuer) for examples and all the configuration options.
