# Contributing

## Prerequisites

- Rust 1.99.1 and Cargo
- Python 3
- Git
- WASM and ARMv7 musl Rust targets

## Build

Keep this project and the Braiins SDK in sibling directories. The build script retrieves the SDK if needed and checks its pinned revision.

```sh
sh tools/build.sh
```

Build output is written to `package/release/`. The pinned SDK revision is `b2c945b5f7a8ea3a7d38a8410a1ca64e8b69b9af`.

For offline dependency resolution, extract `source-dependencies.tar.gz` from the release into the parent directory of this project. It provides `bmc-main/`, `vendor/` and `.cargo/config.toml`. Rust and its required targets must already be installed; the build script's rustup step may need network access.

## Validation

The build script runs service and manifest unit tests. NAR packaging uses the included deterministic Python writer. Additional tests in `tests/` cover shared catalogs, HTTP access control, widget interaction, layout and QR payloads. Widget tests require Python packages `wasmtime`, `Pillow` and `zxing-cpp` as applicable.

Verify playback, touch controls and RGB behavior on a physical Deck before publishing a release. CI builds are not a substitute for device testing.

## Pull requests

Describe the behavior being changed and how it was tested. Preserve all five interface languages and the single multilingual NAR package. Do not embed device IP addresses, passwords, private station credentials or local state files.

Original contributions are MIT-licensed. Preserve third-party copyright and license notices.

Web DOM tests use Node.js and linkedom (`npm install --prefix /tmp/radio-dom linkedom`, then `NODE_PATH=/tmp/radio-dom/node_modules node tests/web_dom.cjs`). These exercise handlers without browser rendering.
