# Verification — 0.7.1

## Automated checks

- Rust service unit tests: PCM levels, local-alarm process matching, URL validation, child-process cleanup, donation persistence, network detection and controls.
- Official pinned SDK manifest parser.
- Real HTTP service tests: optional passwords, shared groups, revisions, rename, sleep and persistence.
- MP3 integration tests using real FFmpeg and a paced sink in place of the physical speaker: upload, limits, nested folders, playlists, pause/resume, next/end, volume/mute continuity and interrupted uploads.
- Network demo test: a real HTTP server and FFmpeg decode without saving the MP3 on Deck storage; last-track recall and alarm reservation timing.
- Clock bridge shell test with isolated paths, mocked audio binaries and a mocked `/proc` entry: reservation precedes playback and a duplicate request preserves the playing sound.
- Actual compiled WASM with mocked host APIs: five languages, four viewports, radio/music navigation, station publishing and player controls.
- Serialized UI geometry inspection: visible controls are at least 48 px and do not overlap. Scroll content is clipped to its viewport.
- Web DOM tests: login state, password changes, folders, file movement and playlists. These do not replace a real browser rendering test.
- NAR round-trip and package structure inspection.

## Running tests

Build the debug service and release widget first. Package widget assets with `python3 package.py`. Python WASM/layout tests require `wasmtime` and `Pillow`; web DOM tests require Node.js and `linkedom`.

```sh
cargo test --manifest-path service/Cargo.toml
cargo test --manifest-path manifest-check/Cargo.toml
python3 tests/service_web.py
python3 tests/music_library.py
python3 tests/release071.py
python3 tests/clock_bridge.py
python3 tests/smoke.py
python3 tests/layout_preview.py
node tests/web_dom.cjs
```

Run service tests sequentially because they use fixed local ports. The test PCM sink intentionally simulates playback pacing; it does not validate the Deck sound driver.

## Hardware checks still needed

Physical swipe behavior, sustained meter FPS and CPU/Wi-Fi impact, real speaker routing, native alarm volume and end-to-end RoliD Clock gong timing have not been verified on the target Deck for this release. A 100 ms request interval is a target, not a measured hardware frame rate. Firmware revisions may behave differently.
