<div align="center">

# RoliD Radio
### Internet radio for Braiins Deck

**Touch controls · Shared station libraries · Remote web control**

[Download](https://github.com/MrRolid/Rolid-Radio-for-Braiins-Deck/releases/latest) · [Installation](#installation) · [Web panel](#remote-web-panel) · [License](#license)

</div>

Turn your Braiins Deck into an internet radio with a compact touch interface and a browser-based control panel. Play MP3 streams, use AAC with a compatible FFmpeg installation, and keep your station library shared across scenes.

One widget package includes **English, Czech, German, Spanish and Japanese**. English is the default.

## Features

- **Large touch controls** for playback, volume and mute.
- **Adaptive presets:** five buttons in Small, eight in Medium and twelve in Full layouts.
- **Shared station libraries** with editable regional starter lists and a custom USER bank.
- **Remote web control** with station editing, reordering, JSON import/export and playback controls.
- **Sleep timer** with 15, 30, 60, 90 and 120-minute options.
- **Audio meters and stream diagnostics** when using FFmpeg.
- **Optional RGB notifications** for changes made through the remote panel.
- **Bitcoin support screen** with an address and scannable QR code.

## Requirements
<img width="1280" height="480" alt="image" src="https://github.com/user-attachments/assets/cf02e64f-2309-48e3-9741-4d1c48876f19" />

| Component | Requirement |
| --- | --- |
| Device | Braiins Deck, ARMv7 |
| Firmware | Custom `.nar` widget installation and a compatible Braiins widget SDK runtime |
| Service installation | SSH access as `root` |
| MP3 playback | `madplay` and `wget` installed on the Deck |
| AAC playback and meters | Compatible `ffmpeg` and `aplay` installed on the Deck |
| Network | An internet connection for streams; a local connection for web control |

The release contains a display widget and a companion audio service. Both are required. Decoder packages are not bundled. AAC requires an FFmpeg build that supports the selected stream and the Deck's firmware; the installer does not modify package feeds or install a decoder.
<img width="1280" height="480" alt="image" src="https://github.com/user-attachments/assets/b843a050-4bad-45f6-b68f-df352027edc5" />

## Installation

Download `rolid-radio-0.5.3.nar` and `rolid-radio-service-0.5.3.tar.gz` from [Releases](https://github.com/MrRolid/Rolid-Radio-for-Braiins-Deck/releases). Replace `DECK_IP` below with your device's IP address.

1. **Install the audio service.** Open a terminal in the directory containing the downloaded service archive:

   ```sh
   ssh root@DECK_IP 'mkdir -p /tmp/rolid-radio-053 && tar -xz -C /tmp/rolid-radio-053 && sh /tmp/rolid-radio-053/install.sh' < rolid-radio-service-0.5.3.tar.gz
   ```

   Keep the web access key printed by the installer.

2. **Install the widget.** In the Deck's web interface, open **Display Widgets → Install Widget**, upload `rolid-radio-0.5.3.nar`, and add **RoliD Radio** to a scene. Choose a station and press Play.

### Updating

Use the same installation procedure to update the service and widget. Station libraries, volume preferences and the device's donation address are preserved. Playback is stopped after the service restarts. An existing compatible FFmpeg installation can be kept.

## On-device controls

| Layout | Presets | Navigation |
| --- | ---: | --- |
| Small | 5 | Station list and settings beside Play/Stop |
| Medium | 8 | Station list, settings, previous/next station |
| Full | 12 | Station list, settings, previous/next station |

The first entries in the active station library become the preset buttons. Use the station list to browse the remaining entries. Empty preset slots are disabled.

Open **Settings** to select a station bank, change the display language, configure RGB brightness, set a sleep timer or open the BTC support screen. **INFO** shows stream diagnostics and the remote panel address.

## Remote web panel

Open **Settings → INFO** on the Deck to find its current address, then visit:

```text
http://DECK_IP:18766
```

Sign in with the access key printed during installation. To retrieve it again:

```sh
ssh root@DECK_IP 'cat /mnt/data/rolid-radio/web-token'
```

The service detects the device's IPv4 address from its network interfaces, preferring active Wi-Fi over Ethernet, and refreshes it every five seconds. No personal IP address is embedded in the package.

The panel provides playback, mute, volume and sleep controls, a shared station editor, diagnostics, an RGB test and the BTC support screen. Its interface language can be selected independently of the widget language.

**Local network access only:** the panel uses HTTP and an access key. Do not expose port `18766` directly to the internet. The widget's internal API listens on loopback port `18765`.

## Station libraries

Editable starter banks are provided for English, Czech, German, Spanish and Japanese listening, alongside a shared **USER** bank. Changes saved in the web panel apply to all radio scenes on the same Deck. Each bank retains its own station list.

Use the web panel to add, rename, reorder or remove stations, or import/export a JSON list. Supply direct HTTP(S) stream URLs. Station availability and bitrate are controlled by the broadcasters.

The Deck's native configuration dialog also offers a **station draft**. To replace the shared USER bank with that draft, increase **Apply station list** by one. Editing the draft alone does not publish it. The native dialog does not mirror the current web library; use the web panel to edit the live shared list.

## RGB notifications

With **RGB notification** enabled, remote changes trigger a short warm-orange pulse. Brightness is adjustable from 0–30%. Use **RGB TEST** in the web panel to request an indication.

The radio widget must be visible. Enable RGB notifications in one widget instance to avoid competing requests. System or preview lighting can take priority. Rapid changes may merge into a single indication.

RGB notifications indicate control activity; they are not a music visualizer.

## Stream information

With FFmpeg, the interface displays measured audio levels and available decoder information. Diagnostics include session time, PCM bytes, data age and reconnect count. The web panel also exposes available FFmpeg progress and error details.

Meters are informational and are not guaranteed to be synchronized precisely with the speaker. With madplay, MP3 playback is available without AAC or PCM meters.

## Support the project

Open **BTC** in the widget settings or scroll to **BTC / SUPPORT** in the web panel to display the donation address and QR code.

Each Deck chooses one configured address and retains it across ordinary updates and scene changes. The widget and web panel display the same address. Donations are optional; all features remain available without a donation.

<img width="246" height="246" alt="image" src="https://github.com/user-attachments/assets/8dc82d20-d3cb-4799-9209-a9be1a441fc2" />
bc1qg24xphhjk05l7xaqfzr4w9nhgje39jm46xfjhv


## Compatibility notes

- Direct MP3/AAC HTTP(S) streams are supported; M3U, PLS and HLS playlists are not.
- Track titles, artist metadata, spectrum analysis and cache-fill percentages are not provided.
- Changing volume restarts the decoder and may briefly interrupt playback.
- The radio does not automatically coordinate speaker access with other audio services.
- The native Deck configuration dialog cannot display a live service-generated URL or QR code. Use the widget's INFO and BTC screens or the remote panel.
<img width="876" height="905" alt="image" src="https://github.com/user-attachments/assets/aca95d0e-a458-4d6b-b95d-0598ecc808fb" />

## Build from source

See [CONTRIBUTING.md](CONTRIBUTING.md) for prerequisites and build instructions. Release assets include dependency sources and checksums.

## Current behavior and limitations
Alarm notifications do not currently interrupt or lower the volume of radio playback. When an alarm goes off, an on-screen notification and an RGB light alert are shown, but the radio continues playing at its current volume.

Adding a station currently requires a direct MP3 or AAC stream URL. Playlist links are not supported yet.

## Roadmap
The following features and improvements are planned:

Alarm integration: connect the radio with RoliD Clock for Braiins Deck, allowing an alarm to start radio playback.

Alarm audio handling: address the current limitation where radio playback continues unchanged when an alarm goes off.

Playlist support: allow stations to be added using playlist links instead of requiring a direct stream URL.

Local music playback: add an MP3/AAC player for locally stored music, with a planned storage limit of 1 GB due to the device’s limited storage capacity.

Home NAS support: add the ability to play music stored on a home NAS.


## License

Original RoliD Radio source code is available under the [MIT License](LICENSE).

The compiled widget incorporates the Braiins SDK and is distributed under **GPL-3.0-or-later**. Japanese font assets retain their **SIL Open Font License**. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for the scope of each license and corresponding source information.

---

An independent community project for Braiins Deck. Not an official Braiins product.
