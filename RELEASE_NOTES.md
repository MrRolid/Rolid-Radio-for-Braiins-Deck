# RoliD Radio 0.7.1

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

Test swipe gestures, sound timing, native-alarm loudness and sustained meter performance on the actual Deck. Native alarm loudness uses the Deck system sound setting; this update does not override it. If it remains quiet, capture the active `madplay` arguments using `tools/alarm-diagnostics.sh` during the alarm. The radio volume filter affects its own stream only.
