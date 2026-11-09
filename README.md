# RoliD Radio for Braiins Deck

Internet radio and a personal MP3 library for the Braiins Deck speaker. One multilingual widget supports English (default), Czech, German, Spanish and Japanese.

## Features

- MP3/AAC internet radio, shared editable station groups and 5/8/12 presets for small/medium/full layouts.
- Optional station logos, volume, mute, sleep timer and PCM stereo meters.
- MP3 uploads, nested folders, metadata editing, ordered playlists, shuffle and repeat.
- Swipe-ready library browsing on the Deck; a library pane occupies the right third in medium/full layouts.
- Radio/Music navigation leaves audio playing; Music Play recalls the last track.
- HTTP(S) network MP3 playback, including a self-contained demo.
- Remote web control panel, optional password and persistent per-device Bitcoin donation address with QR code.
- Playback interruption for local alarm sounds, with automatic resumption.

## Install or update

1. From `package/release` on your Linux computer, install the service. Replace `<Deck-IP>` with the address of your own Deck:

   ```sh
   ssh root@<Deck-IP> 'mkdir -p /tmp/rolid-radio-071 && tar -xz -C /tmp/rolid-radio-071 && sh /tmp/rolid-radio-071/install.sh' < rolid-radio-service-0.7.1.tar.gz
   ```

2. In the Deck web administration, use **Install Widget** to upload `rolid-radio-0.7.1.nar`. Open `http://<Deck-IP>:18766` for remote controls. The actual address also appears in the widget's Settings → INFO panel.

Requires ARMv7 Braiins Deck with `madplay`; local/network music, AAC and meters require FFmpeg and `aplay`. Existing music, playlists, station edits and donation identity are preserved. The installer does not install FFmpeg or change firmware.

## Password

The default password is empty and protection is off. The web panel's security section can set, show, change or disable it. An empty password disables protection. Without protection, anyone able to reach the panel on your network can control playback and modify the library.

Previously saved custom password settings remain intact. An older installer-generated `web-token` by itself is ignored. The native widget configuration offers password settings with an explicit Apply counter; Settings → INFO → PASSWORD on the display shows the live value. The static configuration form cannot automatically show a live value from the service. Changing the Apply counter is required to publish form edits.

## Stations and logos

Use **+ Group** and **Rename group** in the panel for lists such as Jazz, Rock or News. Names and order are editable; Save publishes the group for all scenes. The original language groups are editable starter lists. The widget's title opens the group selector; the list button opens stations.

Choose a PNG, JPEG or WebP in a station row to create a 48×48 PNG logo, then Save. **Text** removes the logo. Source uploads are limited to 2 MiB; converted logos to 8 KiB each and 32 KiB per group. Use images you have permission to redistribute.

## Music library

Create a genre folder, select it, then create an artist folder inside it. Upload or move tracks into the selected folder. Playlists reference existing files; they do not duplicate storage. Library storage is `/mnt/data/rolid-radio/music`: 1 GiB total, 128 MiB per uploaded file, at most 500 tracks and a 512 MiB free-space reserve. Metadata edits affect the library index, not MP3 tags.

Deleting the actively selected track requires ending music playback first. Radio/Music navigation alone does not stop it. Play a radio station before deleting that track.

## Network playback demo

1. On a computer or NAS with Python 3, run `python3 demo-network/serve.py`. This serves the included low-level synthetic tone. Use `--directory /path/to/music` to serve a music directory instead. The server is read-only but has no authentication; use it only on your trusted local network.
2. In the panel's music library, enter a title and `http://<PC-or-NAS-IP>:18767/demo.mp3`, then choose **Add network MP3** and Play. The server must remain running and reachable from the Deck.

This is direct HTTP(S) file playback with range support, not an SMB/NFS client. Network files count toward the 500-track index limit but do not consume local music storage. A dropped network source stops music and reports an error; resume when the source is available.

## Alarms and performance

The service detects local `madplay` sound-file processes and releases its own speaker process. It does not terminate foreign players. An explicit sound reservation lasts at least eight seconds; process detection holds playback for six seconds after the last detection to bridge repeated alarm attempts. This can leave a short silent interval after an alarm ends.

For the recognized RoliD Clock audio service, the installer backs up its CGI as `audio.before-radio-071`, then installs a bridge that reserves the speaker before playing a sound. Unknown audio services are left untouched. Native alarm volume is controlled by the Deck, not the radio volume slider.

Meter messages are separated from the catalog and requested every 100 ms; full status updates run once a second. The PCM path computes levels every 100 ms. No additional decoder is started for visual effects. Timing includes request latency, so 10 FPS is a target rather than a guarantee. Hardware performance and alarm loudness require on-device verification.

## Build and verification

The SDK revision is pinned in `tools/build.sh`. For an offline build, extract `package/release/source-dependencies.tar.gz` beside this project and point Cargo's vendored-source configuration at the extracted `vendor` folder. Rust targets: `wasm32-unknown-unknown` and `armv7-unknown-linux-musleabihf`.

Run `sh tools/build.sh`. See `TESTING.md` for automated checks and remaining hardware checks. Release files contain a SHA-256 checksum list.

## License and support

Original RoliD Radio code is MIT licensed. The SDK and other bundled dependencies retain their own licenses; the linked widget includes GPL-3.0 SDK code. See `LICENSE`, `LICENSES` and `THIRD_PARTY_NOTICES.md` before redistribution.

The donation page displays one randomly selected address persisted for each Deck. [Source and issues](https://github.com/MrRolid/Rolid-Radio-for-Braiins-Deck).
