# nes_emulator

I implemented most of this emulator by reading [Writing NES Emulator in Rust](https://bugzmanov.github.io/nes_ebook/).
I also implemented basic APU support. At least it runs NTSC version of Super Mario Bros.

# How to build and run

If you are using windows, please copy `SDL2.dll` to the root directory of this repository and `SDL2*.lib` to your toolchain's library directory (maybe `%UserProfile%\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\lib`) before running the emulator. You can download them from [the release page of SDL2](https://github.com/libsdl-org/SDL/releases).

```
cargo build --release
target/release/nes_emulator mario.nes
```
