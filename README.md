# `viuer`

![ci](https://github.com/atanunq/viuer/workflows/ci/badge.svg)

Display images in the terminal with ease.

> This clone of the original repository has the following changes:
>
> - Real alpha compositing for the checkerboard background (instead of one bit
>   transparency).
> - Support for pre-multiplied alpha blending (new flag in `Config`).
> - Uses Catmull-Rom for up/downscaling.
> - Updated dependencies.

`viuer` is a Rust library that makes it easy to show images in the terminal.
It has a straightforward interface and is configured through a single struct.
The default printing method is through lower half blocks (`▄` or `\u2585`).
However some custom graphics protocols are supported. They result in full
resolution images being displayed in specific environments:

- [Kitty](https://sw.kovidgoyal.net/kitty/graphics-protocol.html)
- [iTerm](https://iterm2.com/documentation-images.html)
- [Sixel](https://github.com/saitoha/libsixel) (behind the `sixel`
  feature gate)

## Usage

Add this to `Cargo.toml`:

```toml
[dependencies]
viuer = "0.7"
```

For a demo of the library's usage and example screenshots, see
[`viu`](https://github.com/atanunq/viu).

## Examples

```rust
use viuer::{print_from_file, Config};

fn main() {
    let conf = Config {
        // Set offset.
        x: 20,
        y: 4,
        // Set dimensions.
        width: Some(80),
        height: Some(25),
        ..Default::default()
    };

    // Starting from row 4 and column 20,
    // display `img.jpg` with dimensions 80×25 (in terminal cells).
    // Note that the actual resolution in the terminal will be 80×50.
    print_from_file("img.jpg", &conf).expect("Image printing failed.");
}
```

Or if you have a [DynamicImage](https://docs.rs/image/*/image/enum.DynamicImage.html),
you can use it directly:

```rust
// ... `Config` setup

let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(20, 10));
viuer::print(&img, &conf).expect("Image printing failed.");
```

## Docs

Check the [full documentation](https://docs.rs/crate/viuer) for examples and all
the configuration options.
