# Verification — 0.7.2

Actual release-build evidence is in `docs/RELEASE_072.md`. After installing the
0.7.2 service and widget, the owner confirmed the Clock alarm and return to radio.
This hardware confirmation does not establish meter performance.

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

Physical swipe behavior, sustained meter FPS and CPU/Wi-Fi impact,
multi-scene alarm ownership and restart/snooze timing still need
target-device verification. The owner confirmed audible Clock alarm volume at
70% and return to radio after the CGI fixes and again after the 0.7.2 update. Native
firmware alarms were reported working. A 100 ms request interval is a target,
not a measured hardware frame rate. Firmware revisions may behave differently.

## Takeover checks for the next patch

`python3 tests/alarm_diagnostics.py` checks privacy filtering and attenuation capture
without a Deck. `python3 tests/clock_bridge.py` also verifies a duplicate zero-volume
request stops the sound without a second reservation. See
`docs/LOCAL_VERIFICATION_072.md` for this environment's actual results and blockers.
