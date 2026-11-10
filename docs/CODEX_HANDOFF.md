# RoliD Radio — Codex handoff

Latest follow-up: **0.7.2** was prepared locally at the owner's request. Read
`RELEASE_072.md` for current artifact/test status and `LOCAL_VERIFICATION_072.md`
for the on-device diagnosis. The sections below describe the original 0.7.1
handoff. The owner clarified that native alarms work; the quiet alarm was Clock.
Clock volume and return to radio were subsequently confirmed after CGI fixes.

Prepared 2026-11-10. Baseline: **0.7.1**, service API **7**.
Repository intended by owner: https://github.com/MrRolid/Rolid-Radio-for-Braiins-Deck
The ZIP is a source snapshot, not a clone. Its synchronization with GitHub HEAD has not been checked. Compare before replacing an existing checkout; never force-push it over repository history.

## Architecture and code map

| Component | Location | Responsibility |
|---|---|---|
| LCD widget | `src/lib.rs` | UI state, touch commands, status and meter polling, settings, library navigation |
| Drawing/layout | `src/design.rs` | Controls, preset logos/names, colors and dimensions |
| Localization | `src/i18n.rs`, `src/jp.rs`, `translations.json` | Translations and Japanese rendering support |
| Service | `service/src/main.rs` | Playback lifecycle, catalog, commands, local API, web dispatch |
| Library | `service/src/music.rs` | Index, upload, folders, playlists, last-track persistence, network tracks |
| Security | `service/src/access.rs` | Optional password; compatibility fields `key`/`enabled` |
| Alarm monitor | `service/src/alarm.rs` | Poll `/proc` for local madplay sound files |
| PCM meters | `service/src/meter.rs`, `telemetry.rs` | Real PCM levels and FFmpeg progress |
| Remote UI | `service/web/index.html`, `service/src/web.rs` | Embedded web UI/API; compile service after web edits |
| Network address | `service/src/network.rs` | Runtime interface/address detection; no personal IP embedded |
| Clock bridge | `service-package/clock-audio-bridge` | CGI compatible with RoliD Audio V1; reserve speaker before sound |
| Installation | `service-package/install.sh`, `rolid-radio.init` | OpenWrt service installation and recognized Clock bridge backup |
| Manifest | `manifest.json`, `manifest-check/` | Widget settings and official parser validation |
| Packaging | `package.py`, `asset-pack/`, `tools/release.py`, `tools/nar.py` | Assets, launcher, NAR and service tarball |
| Tests | `tests/`, `TESTING.md` | Real decode/API tests, WASM mock-host tests, DOM and geometry checks |
| NAS demo | `demo-network/` | Range-aware Python HTTP server and synthetic MP3 |

## Device and runtime

Known target is ARMv7 `armv7l`, OpenWrt 22.03.4, custom target `stm32mp15/ii3`, package arch `arm_cortex-a7_neon-vfpv4`. Audio is BMC100-MAX98357A, `plughw:0,0`; `/usr/bin/aplay`, `/usr/bin/madplay` and FFmpeg were installed by the owner. Do not assume the same packages on a fresh device.

Local API: `127.0.0.1:18765`; local command authorization is `Rolid-radio-v1` (an internal protocol constant, not the user's password). Remote HTTP panel: port 18766. Clock CGI: loopback port 18764, `Rolid-local-v1`. Demo HTTP server: 18767 on a PC/NAS.

Persistent radio root: `/mnt/data/rolid-radio`. Contains `state.json`, `catalog-store.json`, `web-access.json`, donation identity and `music/`. Legacy `web-token` alone no longer enables protection. `music/library.json` stores folders, playlists and track metadata; tracks have opaque IDs. `music/last-track` remembers the last track ID. Network tracks have `source` URLs and no local MP3 payload. Library limits: 1 GiB local, 128 MiB/file, 500 tracks, 100 folders/playlists, reserve 512 MiB.

Clock root: `/mnt/data/rolid-audio`; its CGI is `www/cgi-bin/audio`. The 0.7.1 installer replaces only a file containing `ROLID_AUDIO_SERVICE_V1`, backs it up as `audio.before-radio-071`, and keeps the original sound files. The complete current Clock widget/audio project is **not** included in this radio handoff. Retrieve it from the owner/device before changing its scheduling engine.

## Implementation and constraints

FFmpeg produces 48 kHz stereo S16_LE PCM; `aplay` provides backpressure. RMS/peak values are aggregated every 100 ms. `/levels` returns meter data only; the widget/web request it at 100 ms intervals after responses. Full status is about once a second. Physical timing includes latency and is not a measured 10 FPS guarantee. Avoid large catalog serialization at meter frequency. Check repeated scene/widget instances for duplicate polling.

UI screen choice is independent from service source mode. The `radio_mode` API command still explicitly stops music for existing tooling; navigation buttons no longer send it. Music Play uses `music_resume` and recalls the last track. A new source starts that track from its beginning, not a stored seek position across reboot.

Alarm detection scans `/proc` every 50 ms for madplay with an absolute `.mp3` path; it cannot distinguish every possible firmware audio implementation. Detection extends hold by six seconds; explicit `duck` reserves at least eight seconds and cannot be shortened by the monitor. The Clock CGI calls `--duck` before madplay and ignores a repeat request while the identical sound process is still present. This is process matching, not a complete scheduler-level alarm protocol. It can leave 6–8 seconds of silence after a short sound.

Native alarm loudness uses the firmware's sound-volume configuration. Radio uses a per-process FFmpeg volume filter and does not change the system mixer. The owner's report of a quiet native alarm is unresolved; capture actual arguments/settings before deciding on a fix. `tools/alarm-diagnostics.sh` is read-only. Do not log password configuration while diagnosing audio.

Station logos are PNG hex in catalog entries (48×48 from web conversion, validation up to 96×96, 8 KiB each, group 32 KiB). Widget bitmap cache currently lives for the widget instance. Review cache growth when repeatedly changing logos. New custom group names persist; initial language groups remain selectable and may be recreated after a rename. Assess whether that behavior matches the desired UX.

Native widget configuration is static: it holds a station draft and password form, not a live mirror. Applying uses explicit counters to avoid overwriting shared changes on every load. Live password/address is displayed in the LCD INFO panel and web panel. Do not promise dynamic IP links in static manifest Markdown without host support.

HTTP(S) direct MP3 is implemented and tested. SMB/NFS, discovery, credentials for NAS shares, library scans and reconnect/resume after NAS outages are not implemented. Authentication for the demo server is absent; use on a trusted LAN.

## Build baseline

Build toolchain used: Rust **1.99.1**, Linux x86_64 host. Targets: `wasm32-unknown-unknown`, `armv7-unknown-linux-musleabihf`; ARM linker `rust-lld`. SDK revision: `b2c945b5f7a8ea3a7d38a8410a1ca64e8b69b9af`.

This handoff adds `tools/prepare-codex.py` to restore sibling SDK/vendor folders without replacing existing directories. It creates a local ignored `.cargo/config.toml`; Rust/toolchain installation remains separate. `tools/build-offline.sh` builds from vendored sources, with targets already installed. The handoff does not change the shipped 0.7.1 binaries.

Requirements: Rust/rustup, native C compiler/linker, Python 3.12+, FFmpeg for service tests, Python `wasmtime` and `Pillow` for widget/layout tests, Node and `linkedom` for web DOM tests. Offline here refers to Cargo dependencies, not installation of these host tools. Browser rendering needs a separately available browser.

## Evidence and remaining work

Previously passed: 10 service unit tests; official manifest parser; HTTP/password/catalog tests; real FFmpeg with a paced mock speaker; upload/library/playlist/persistence tests; real HTTP MP3 demo; explicit alarm hold test; compiled WASM host-mock tests in 317×238, 638×238, 638×480 and 1280×480; visible touch geometry ≥48 px; DOM tests; NAR round-trip/package validation. Clock CGI tests use mocked audio binaries **and mocked `/proc`**, not the physical device.

Not confirmed by owner: 0.7.1 installation, physical swipe, sustained meter FPS/CPU/Wi-Fi behavior, real Clock gong behavior, native alarm volume. The previous failures must remain open until reproduced and verified on hardware. Public README must not imply they are definitively fixed.

Recommended next task: `NEXT_TASK.md`. Other review items: station-name fitting for long names; untranslated newer UI strings; active-track deletion UX; duplicate metric polling across scenes; bank rename semantics; bounded logo cache; explicit Clock alarm lifecycle versus polling; native password character restrictions.

## Release discipline

Current release files remain in `package/release`; checksums are provided. `tools/release.py` regenerates NAR/service tar and checksums but **does not regenerate the source ZIP**. The embedded 0.7.1 source ZIP is the original release source, before handoff documentation/helpers. For continued development, the project root in this handoff is authoritative. Do not overwrite newer working files by unpacking that older source ZIP.

No GitHub upload, tag, push or device deployment was performed as part of this handoff. The bundle contains no repository history, installed toolchain, credentials, personal music or live device configuration. Preserve MIT and dependency notices; linked SDK includes GPL-3.0 components.

## Takeover follow-up (2026-11-10)

See `LOCAL_VERIFICATION_072.md` for new local evidence, environment blockers and the
Clock duplicate-volume-zero fix prepared for 0.7.2. Shipped artifacts and version
strings remain at the preserved 0.7.1 baseline until a complete release can be built.
