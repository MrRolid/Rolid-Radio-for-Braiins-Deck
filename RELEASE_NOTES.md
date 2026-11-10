# RoliD Radio 0.7.2

- Fix installation of the recognized Clock audio CGI: use mode 755, required by
  uhttpd, instead of mode 700, which caused HTTP 403 before the CGI could run.
  The server remains loopback-only. Updates preserve the previous CGI as
  `audio.before-radio-072` without replacing the earlier 0.7.1 backup.
- A zero-volume request now stops an already playing identical Clock sound;
  duplicate nonzero requests still leave that sound playing without restarting.
- Preserve user recordings, schedules, radio/music data, password settings and
  donation identity. No system mixer, volume mapping or sound-file boost changes.
- Add privacy-filtered, read-only sound diagnostics and offline build preparation.
- Rebuild the multilingual widget and ARMv7 service as 0.7.2, retaining API 7 and
  the pinned SDK revision and dependency licenses.

Local service/API, real FFmpeg playback, widget and package checks are documented
in `docs/RELEASE_072.md`. The owner confirmed correct Clock alarm volume at 70%
and return to radio on the Deck, first with the CGI fixes on 0.7.1 and then
again after installing the complete 0.7.2 service and widget.

Native firmware alarms were reported working; the quiet-alarm report concerned
Clock. Sustained meter rate, physical swipe, multi-scene alarm ownership and
restart/snooze timing remain outside that hardware confirmation.

# RoliD Radio 0.7.1 (historical)

- Optional **password**, empty and disabled on a new installation. A legacy auto-generated `web-token` alone no longer enables protection. An explicitly saved password configuration remains intact. An empty password disables protection.
- Radio/Music navigation preserves current playback. Music Play recalls the last selected track, including after a restart. Selecting a new radio station still switches the audio source.
- Native scroll containers for the music library and radio groups. Nested genre/artist folders and playlists share the existing persistent library.
- Editable, named radio groups and optional station logos uploaded in the web panel. Logo images are converted to small PNGs. Preset labels use a 16 px target size, with wrapping and fitting where necessary.
- Fullscreen music has vertical stereo meters at the left; medium/full retain the library in the right third.
- Compact `/levels` responses run separately from the one-second full status refresh. Meter polling targets 10 Hz and does not change the audio processing rate. Physical Deck FPS, CPU and Wi-Fi impact still need device verification.
- Network MP3 library entries play direct HTTP(S) files without storing their contents on the Deck. Includes a range-aware Python demo server and synthetic MP3. SMB/NFS mounting and NAS discovery are not implemented.
- Alarm monitoring no longer shortens an explicit speaker reservation. It bridges short gaps between alarm attempts before resuming playback.
- A recognized RoliD Clock audio CGI receives a backed-up bridge that pauses Radio before sound playback and avoids restarting an already playing identical sound. Sound files and configured sound volume remain unchanged.

## Device verification still required

Test swipe gestures, sound timing and sustained meter performance on the actual Deck. The owner subsequently clarified that native firmware alarms were working; the quiet alarm was in Clock. See the 0.7.2 notes above for the later diagnosis and confirmation.
