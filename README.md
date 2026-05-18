# Hidden Character Cleaner

Native desktop app for removing invisible Unicode formatting and control characters from pasted text. It is built with Rust and `egui`/`eframe` so it can be packaged for Windows, Linux, and macOS.

## Build

Install Rust, then run:

```bash
cargo build --release
```

The executable is created under `target/release/`.

## Run

For development:

```bash
cargo run
```

For normal macOS double-click launching without Terminal:

```bash
./scripts/package_macos_app.sh
open "dist/Hidden Character Cleaner.app"
```

You can also double-click `dist/Hidden Character Cleaner.app` in Finder.

## Test

```bash
cargo test
```

## Package for Common Platforms

macOS app bundle:

```bash
./scripts/package_macos_app.sh
```

Windows x86_64 `.exe` from macOS:

```bash
./scripts/build_windows.sh
```

Linux x86_64 binary from macOS:

```bash
./scripts/build_linux_x86_64.sh
```

Windows builds require `mingw-w64`. Linux x86_64 builds use `cargo-zigbuild` and `zig`.

On Windows, the executable is configured with `windows_subsystem = "windows"` so launching it from Explorer opens only the GUI, not a console window.

For polished installers, use a packaging tool such as `cargo-bundle`, `cargo-packager`, or `cargo-dist`.

## Output Formats

The app can save cleaned output as:

- `.txt`
- `.docx` Microsoft Word document

## Language Support

The app detects the system language on startup and supports:

- English
- German

English is used when the system language is not supported. The language can also be changed from the toolbar.

## Em Dash Handling

The `Replace em dashes` option converts common AI-style dash characters to normal dash sequences:

- `—` to `-`
- `⸺` to `--`
- `⸻` to `---`

## GPTZero Helper

The `Copy + Open GPTZero` button copies the cleaned text to your clipboard and opens GPTZero in the default browser. Paste the text manually on the site to use GPTZero's free checker. The app does not scrape or automate GPTZero.

## Development Disclaimer

This software was written with assistance from an AI coding assistant. Review the code and behavior before relying on it for important workflows.

## What It Removes

The cleaner removes common invisible Unicode characters, including zero-width characters, bidirectional formatting controls, byte order marks, soft hyphens, variation selectors, and non-printing controls while preserving ordinary tabs and line breaks.

It is a hidden-character cleaner; it cannot verify or bypass any undisclosed proprietary provenance system.
