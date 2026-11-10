# Local takeover verification — 2026-11-10

This is a chronological record. For the subsequent completed local 0.7.2
build, passed integration/UI/package tests and release artifacts, read
`RELEASE_072.md`. Earlier environment blockers below were resolved for that
build. The 0.7.2 service was subsequently installed with owner authorization;
radio status is playing and Clock HTTP status is ready. The owner installed
the widget manually and confirmed the Clock alarm works after updating.
See the deployment section in RELEASE_072.md.

Baseline remains 0.7.1; next intended release is 0.7.2. No release was generated,
installed or published. Git has no commits and the supplied source tree is untracked;
there is no local history with which to compare the snapshot.

## Changes prepared

- Clock bridge: a repeated request for the same sound with volume zero now reaches
  stop/mute handling instead of returning `playing`. This does not establish a fix
  for quiet native alarms or repeated scheduler requests after a sound has ended.
- Regression test fails against the original bridge and passes against the change.
- Read-only alarm diagnostics now report kernel/OpenWrt release, madplay attenuation
  and filtered service state. Full source URLs, music metadata and network settings
  are omitted. A synthetic proc/status test verifies filtering and attenuation.

## Build and test evidence

Rust 1.99.1 is installed. `tools/prepare-codex.py` restored the supplied dependency
archive in an isolated copy at `/tmp/rolid-work/project` because the original parent
folder is read-only in the sandbox. Existing SDK/vendor/Cargo configurations were
not overwritten. SDK marker matches the pinned revision.

Passed:
- Offline native service build and all 10 service unit tests, with an explicit
  temporary `ROLID_RADIO_DATA_DIR`.
- Official manifest parser test and native asset pack build.
- Clock bridge regression including duplicate, zero-volume and explicit stop.
- `python3 tests/alarm_diagnostics.py`; shell syntax checks.
- All original release SHA256SUMS, exact NAR round trip, and restored NAR structure,
  wrapper, asset verification and permissions checks.

Blocked (not passed):
- WASM and ARMv7 builds fail with E0463: target standard libraries are missing.
  Installing targets into an isolated copy of rustup failed on DNS resolution.
- service_web.py, music_library.py and release071.py were attempted sequentially;
  sandbox socket creation fails with EPERM even on loopback. No HTTP integration
  success or actual audio decoding result is claimed from this run.
- wasmtime is missing; dependency download failed on DNS. New WASM smoke/layout and
  DOM verification are pending; linkedom installation was also attempted.

Source inspection confirms Radio/Music navigation avoids the `radio_mode` command
and Music Play uses `music_resume`; last-track persistence and nested parent folders
remain in place. These observations do not replace runtime or physical swipe tests.

## Hardware still required

## SSH diagnostic follow-up — 2026-11-10

Authenticated SSH access is now confirmed using an owner-installed dedicated key.
Read-only device capture reports kernel 5.10.176, OpenWrt 22.03.4, target
stm32mp15/ii3, and radio 0.7.1 playing. No local madplay process was active
during the capture. The Deck application firmware version remains unconfirmed.
The mixer reports Master 99/99 (100%) on both channels and Speaker on; no mixer
setting was changed. These readings do not establish native alarm loudness.

The installed Clock CGI exactly matched the original bridge (SHA-256
6103b2f6f8d65eea835dfcfda15446f0a386e0629405e70a30d6d483c5555eed).
With owner authorization, the locally tested zero-volume duplicate fix was
deployed after validating the original hash and creating a separate backup at
`/mnt/data/rolid-audio/www/cgi-bin/audio.before-zero-volume.EKaihm`.
Replacement was atomic; CGI syntax and replacement hash were checked before
installation. The installed CGI's read-only status returns HTTP 200 / ready.
No radio binary, sound file, scheduler, credentials or system volume was changed.

Local Clock bridge and privacy-filter diagnostics tests passed again. Physical
stop/mute behavior, repeated interruptions and alarm loudness are still pending
a live alarm capture. The full Clock scheduler source is still unavailable here.

### Earlier capture limitations (before SSH access)

Owner correction during the SSH follow-up: the native firmware alarm is fine.
The quiet alarm report concerns the **Clock widget**, not the native alarm.
Earlier references to a quiet native alarm in this handoff are superseded by
this clarification; do not tune or diagnose native alarm volume for this issue.

Clock-specific read-only evidence: gong.mp3 and alarm.mp3 have identical SHA-256
hashes, both are 47,133 bytes. FFmpeg decodes the alarm as 3.89 seconds with mean
volume -18.3 dBFS and peak -3.2 dBFS. Original Clock CGI and the installed bridge
both map volume to rounded `volume * 40 / 100 - 40` dB (zero is mute).
Thus a request at 50 applies -20 dB, at 70 applies -12 dB, and at 100 applies
0 dB. The actual scheduler request volume has not yet been captured; these
measurements alone do not justify modifying the mapping or boosting the file.

After the owner reported "Clock zvoní", a read-only SSH snapshot and repeated
process/status sampling beginning at 12:52:29 UTC found no madplay process and
radio status remained playing in the observed samples through 12:53:27 UTC.
This does not confirm that Clock was still sounding during sampling, nor that
the custom MP3 service was selected. Capture timing and `use_audio_service`
must be established before attributing this occurrence to the -12 dB CGI path.
Read-only source review used `/home/herdr/rolid-clock` (the owner-provided
`radio-clock` path does not exist). The widget has an alternative SDK audio
path when `use_audio_service` is false. No Clock source was modified.

### Confirmed Clock HTTP permissions fault and audible service test

The current configuration was queried on-device with jsonfilter, restricted to
Clock audio fields: use_audio_service=true, alarm_volume=70, ring_seconds=120.
The installed widget manifest is Clock 0.6.0 and includes use_audio_service;
boolean controls were absent from the owner's copied configuration text.

A loopback HTTP POST received uhttpd's HTML 403 Forbidden, before the CGI ran.
The CGI was mode 700, as installed by the Radio installer. After a separate
on-device backup (`audio.before-http-permissions.nkOJFG`), changing only the
recognized CGI to mode 755 made HTTP GET ?status return ready. The CGI remains
behind a loopback-only server with its local request checks unchanged. No mixer
or audio file changes were made. Radio's source installer now uses mode 755 for
this CGI; release archives were not rebuilt.

A subsequent HTTP POST action=alarm&volume=70 returned 200 OK / accepted.
The owner heard a gong and confirmed the return to radio afterward. This is an
audible hardware test of the service path, not yet a scheduled widget alarm
lifecycle test or confirmation of adequate loudness. Both Clock MP3 files are
identical gongs. The actual -A argument was not sampled during this successful
request. Earlier diagnostic POST attempts using wget's unsupported --header,
nc's unsupported -w, or a prematurely closed nc input did not establish a
playback result. The successful request kept nc input open until the CGI reply.

Local Clock bridge and diagnostics tests and installer/CGI shell syntax checks
passed again. Read-only review of the separate Clock source installer found it
already uses mode 755 and documents uhttpd's S_IXOTH requirement. No edit to that
project was needed or made. Scheduled alarm repeats,
full alarm ownership across scenes and subjective loudness remain unverified.

### Owner confirmation after the permissions fix

After being asked to test an alarm directly in the Clock widget, the owner
confirmed: "je to ok... zvuk ok.. i navrat k radiu". The tested Clock alarm's
audible volume and return to radio are therefore confirmed on the physical
Deck at the reported 70% setting, following the CGI permissions fix. No sound
boost, mixer adjustment or volume mapping change was required. This supersedes
the pending subjective-loudness/widget-alarm notes above for this tested case.

This confirmation does not measure repeat timing, multi-scene ownership,
simultaneous alarms, snooze, restart behavior, or sustained meter performance.
The Deck application firmware version was not obtained; kernel/OpenWrt details
are recorded above. Existing 0.7.1 archives remain unchanged and no new release
was published. The separate Clock source project remains unmodified.

No Deck access was established. Native alarm attenuation, firmware volume setting,
Clock scheduler behavior, physical swipe, sustained meter rate, CPU/memory/Wi-Fi and
audio continuity remain unverified. No system volume was changed. The existing
six/eight-second holds were preserved pending actual device evidence.

For an owner-provided read-only capture while the native alarm sounds:
`ssh root@"$DECK_HOST" 'sh -s' < tools/alarm-diagnostics.sh`
The script reports OpenWrt/kernel information; record the Deck firmware version
shown in administration too, since that may differ from the base OpenWrt release.
