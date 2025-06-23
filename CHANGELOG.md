## next
- remove `lazy_static` dependency in favor of `std::sync::LazyLock`
- MSRV is now 1.80
- Dont Error in kitty if the temporary file has been deleted by the terminal. (Now `KittySupport::Local` is possible again)

## 0.9.2
- Use iterm and sixel in more terminals
- Bump `crossterm` and `sixel-rs` dependencies

## 0.9.1
- Add docs.rs metadata

## 0.9.0
- Remove `image` default features
- Put `print_from_file` behind the new `print-file` feature flag

## 0.8.1
- Revert removed `image` features

## 0.8.0
- Remove unneeded features and update dependencies
- Use Catmull-Rom for up/downscaling
- Add `premultiplied_alpha` Config option

## 0.7.1
- Bump `base64` and `crossterm` dependencies

## 0.7.0
- Update name of temporary files when using Kitty to contain `tty-graphics-protocol`

## 0.6.2
- Upgrade `crossterm` dependency to 0.25
- Check `LC_TERMINAL` env variable when deciding iTerm support

## 0.6.1
- Upgrade `crossterm` dependency
- Move to 2021 Edition

## 0.6.0
- Upgrade `image` dependency

## 0.5.3
- Bump `crossterm` and `console` dependencies

## 0.5.2
- Use iTerm protocol for WezTerm and mintty
- Fix compiler warnings

## 0.5.1
- Fix memory leak when checking for Kitty support not in tty

## 0.5.0
- Upgrade to `crossterm` 0.20
- Remove `ViuError::Crossterm`
- Rename `ViuError::IO` -> `ViuError::Io`
- Change `print_from_file` signature to take `AsRef<Path>` instead of `&str`
- Add carriage return after every line of printed blocks

## 0.4.0
- Experimental Sixel support
- Remove `resize` Config option
- Change `Printer` trait function signatures
- Improve test suite
- Major refactor of `BlockPrinter`

## 0.3.1
- Make `ViuResult` public

## 0.3.0
- Add iTerm support and `use_iterm` Config option
- Add support for remote Kitty printing, through escape sequences
- Rename `has_kitty_suport` to `get_kitty_support`
- Remove `kitty_delete` Config option

## 0.2.0
- Add support for local Kitty printing
- Add `restore_cursor`, `use_kitty` and `kitty_delete` Config options

## 0.1.0
- Introduce block printing
