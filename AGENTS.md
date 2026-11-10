# Instructions for the next Codex agent

## Start here

Read `docs/CODEX_HANDOFF.md`, `README.md`, `TESTING.md` and `docs/NEXT_TASK.md` before editing. This is a handoff of delivered version **0.7.1**, not a new implementation. The existing installable NAR and service archive are the baseline. Do not claim hardware tests passed: the author has not yet confirmed 0.7.1 on the Deck.

## Work with the owner

- Respond in Czech, directly and without motivational filler. Public README and code documentation stay in English. Preserve all five UI languages: en (default), cz, de, es, ja.
- Give no more than two user-facing procedure steps at a time. Use `mcedit` for any manual editing instructions; preferably implement changes yourself.
- State an honest confidence estimate and distinguish local tests from physical Deck results.
- Do not ask again about settled product requirements listed below. Keep working through build and tests; report real blockers precisely.
- Do not publish, push, deploy to the Deck, or alter system sound settings unless the current task authorizes it. Local implementation and reversible checks are expected.
- Never print, commit or package passwords, keys, user library contents or personal network configuration. Use a configurable `DECK_HOST`; no hard-coded personal IPs.

## Product constraints

- One multilingual `.nar`. Next patch is normally 0.7.2, but confirm the intended number from the current task before releasing; do not jump to 0.8.0 automatically.
- Password protection is optional and empty/off by default. Preserve explicit saved user settings. Never reintroduce a mandatory generated access key. Keep existing API field names compatible if changing labels.
- Navigation between Radio and Music must not change audio playback. Explicit Play/station/track selection may change it. Remember the last music track.
- Shared, editable radio groups and library across scenes. 5/8/12 preset buttons in small/medium/full. Medium/full retain side arrows and right-third library browsing.
- Preserve nested genre/artist folders, playlists, optional station logos and the stable per-device donation address/QR.
- Large touch targets (minimum 48 px), minimal industrial design. Meters must remain subordinate to stable audio and Wi-Fi.
- No simulated beat detection. Existing RGB is remote-action feedback, not music visualization.
- Alarms take audio priority and playback resumes afterward. Never kill unrelated audio processes or subscribe to an alarm overlay merely as a passive observer: that may suppress firmware behavior.

## Engineering requirements

- Preserve the pinned SDK revision and its licenses. Original code is MIT; linked SDK/dependencies have their own licenses, including GPL-3.0. Do not relabel the combined artifact as MIT-only.
- Never reset device music, station catalogs, donation identity or credentials during an update. Back up recognized files before altering another service. Do not overwrite unknown services.
- Use the supplied source-dependency archive for reproducibility. Build WASM and ARMv7 service separately; the Deck cannot run an x86 host binary.
- Test source/API changes meaningfully. Run service integration tests sequentially (fixed ports). Do not run the service with its default production data directory on a development computer.
- Maintain release versions consistently in manifest, Cargo packages/locks, web UI, service status and notes. Rebuild both artifacts when both components change.
- Validate the restored NAR: exactly one widget directory under `lib/bmc-widgets`, with manifest, wrapper, WASM and assets. Never package raw `bin` and manifest directly under `bmc-widgets`.
- `tools/release.py` replaces old NAR/service archives in its output directory. Archive an existing release before rebuilding. The source ZIP is not refreshed by that script; regenerate it deliberately and refresh SHA256SUMS.
- Update handoff notes and test evidence at the end of a substantial change. Do not claim 10 FPS or resolved alarm loudness without device measurements.
