# `viuer`

![ci](https://github.com/atanunq/viuer/actions/workflows/ci.yml/badge.svg)

Display images in the terminal with ease.

`viuer` is a Rust library that makes it easy to show images in the terminal.
It has a straightforward interface and is configured through a single struct.
The default printing method is through lower half blocks (`▄` or `\u2585`).
However some custom graphics protocols are supported. They result in full
resolution images being displayed in specific environments:

- [Kitty](https://sw.kovidgoyal.net/kitty/graphics-protocol.html)
- [iTerm](https://iterm2.com/documentation-images.html)
- [Sixel](https://github.com/saitoha/libsixel) (behind the `sixel`
  feature gate)

For a demo of the library's usage and example screenshots, see
[`viu`](https://github.com/atanunq/viu?tab=readme-ov-file#examples).

## Usage

With the default features, only [image::DynamicImage](https://docs.rs/image/latest/image/enum.DynamicImage.html) can be printed:

```rust
use viuer::{print, Config};

let conf = Config {
    // Start from row 4 and column 20.
    x: 20,
    y: 4,
    ..Default::default()
};

let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(20, 10));
print(&img, &conf).expect("Image printing failed.");
```

And with the `print-file` feature, `viuer` can work with files, too:
```rust
use viuer::{print_from_file, Config};

let conf = Config {
    // Set dimensions.
    width: Some(80),
    height: Some(25),
    ..Default::default()
};

// Display `img.jpg` with dimensions 80×25 in terminal cells.
// The image resolution will be 80×50 because each cell contains two pixels.
print_from_file("img.jpg", &conf).expect("Image printing failed.");
```

## Docs

Find all the configuration options in the [full documentation](https://docs.rs/viuer/latest/viuer/).
