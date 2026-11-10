# RoliD Radio 0.7.2 — release evidence

Prepared locally on 2026-11-10 at the owner's request. No Git commit exists in
this checkout; source files are untracked. No tag, push or public release was
performed. Subsequent owner-authorized service deployment is recorded below.

## Delivered changes

- The Radio installer now installs the recognized Clock CGI with mode 755.
  Mode 700 caused uhttpd to reject requests with HTTP 403 before running the CGI.
  A separate `audio.before-radio-072` backup preserves the previous CGI; older
  backups and user recordings are retained. Unknown services remain untouched.
- A repeated request for the same Clock sound with volume zero now reaches the
  stop/mute path. Nonzero duplicates do not restart a currently playing sound.
- Privacy-filtered, read-only audio diagnostics and offline build preparation
  from the supplied SDK/vendor archive are included in the source snapshot.
- Widget manifest, Cargo package/lock versions, embedded web UI and service
  status consistently use 0.7.2. Service API remains 7. No mixer, sound file,
  saved user setting or volume-curve change is included.

## Build

Both components were rebuilt from the supplied dependency archive in an isolated
working copy, with Rust 1.99.1, `--offline --locked`, and SDK revision
`b2c945b5f7a8ea3a7d38a8410a1ca64e8b69b9af`. Missing WASM/ARM target libraries
were installed separately before the offline build. Existing sibling SDK/Cargo
configuration was not overwritten. Unit tests used a temporary data directory.

- Widget: wasm32-unknown-unknown, optimized release, asset pack generated anew.
- Service: armv7-unknown-linux-musleabihf, rust-lld, optimized release. ELF and
  ARM attributes confirm a static 32-bit ARM EABI5 executable, ARMv7 and VFP ABI.
- MIT original-code license and SDK/dependency notices, including GPL-3.0, remain
  included. The combined widget is not described as MIT-only.

## Local checks passed

- All 10 service unit tests and the pinned SDK's manifest parser test.
- Sequential service_web.py, music_library.py and release071.py regression
  suites against the rebuilt 0.7.2 native test binary and temporary data.
  These exercised HTTP/password/catalog behavior, real FFmpeg decoding, uploads,
  nested folders/playlists, persistence, pause/resume, volume/mute continuity,
  HTTP network MP3, last-track recall and alarm reservation hold.
- Compiled 0.7.2 WASM smoke tests for en/cz/de/es/ja, 5/8/12 presets, four
  viewport sizes, navigation/playback controls, shared groups and folders.
- Serialized layout checks for bounds, non-overlap and visible targets >=48 px.
  These use a mock host and Pillow, not the Deck renderer or physical touch.
- Web DOM tests using linkedom; Clock CGI regressions including duplicate mute;
  diagnostic privacy filtering; installer/CGI/diagnostics shell syntax.
- Independent NAR restore and exact byte-for-byte round trip. Restored package
  has exactly one widget directory under lib/bmc-widgets, with the SDK wrapper,
  matching manifest, valid WASM, verified assets and executable permissions.
- Service tar membership, ARM executable, installer/CGI contents and equality
  with the built sources. Original 0.7.1 archive checksums remain valid.

## Physical Deck evidence and limits

Kernel 5.10.176, OpenWrt 22.03.4, target stm32mp15/ii3. The application firmware
version was not obtained. At test time the Radio binary was still 0.7.1.

After backing up and applying the CGI fixes to that installation, a loopback
HTTP request returned 200 / accepted. The owner then tested the Clock widget
alarm and confirmed correct sound at 70% and return to radio. Both installed
Clock MP3 files contain the same gong. Native firmware alarms were reported
working; the quiet-alarm report concerned Clock. No mixer adjustment or audio
boost was needed. See LOCAL_VERIFICATION_072.md for diagnostic details.

Physical swipe, sustained meter
FPS/CPU/Wi-Fi/audio continuity, multiple scenes, simultaneous alarms and
snooze/restart/unload timing remain unverified. No 10 FPS guarantee is claimed.

## Artifacts

`package/release` contains the 0.7.2 multilingual NAR, ARMv7 service tarball,
source ZIP, original source-dependencies.tar.gz and SHA256SUMS. The source ZIP
contains source/docs/licenses, excludes build trees, archives and local/private
configuration, and uses a top-level rolid-radio directory. Put the separately
supplied dependency archive in its package/release folder before running
tools/prepare-codex.py in a fresh parent directory.

The original 0.7.1 release, including its source ZIP and checksum list, is
preserved separately in `package/archive/0.7.1`.

## Subsequent authorized service deployment

At the owner's explicit request, the released ARMv7 service archive was installed
on the Deck using its supplied installer. The installer completed, service
status reports version 0.7.2, and the previously playing radio station was resumed
from the saved device settings. The API reports mode=radio and status=playing.
Clock's loopback HTTP readiness check returns ready after installation. Private
installer output remains only on the Deck; no credentials or source URLs were
printed. The installer retains the previous service binary and recognized Clock
CGI backup. No system volume settings were changed.

The owner installed the 0.7.2 NAR manually and then confirmed that the Clock
alarm was working. Thus the service and widget update and the tested Clock
alarm/return-to-radio scenario are confirmed on the physical Deck. Multi-scene,
snooze/restart and performance limits above still apply.

To refresh an already built release's source snapshot, run
`python3 tools/source_archive.py` deliberately, then recompute SHA256SUMS for
all non-hidden release files except SHA256SUMS itself. tools/release.py still
replaces NAR/service artifacts and does not regenerate the source ZIP.
