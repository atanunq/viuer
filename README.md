Code Relay:

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
