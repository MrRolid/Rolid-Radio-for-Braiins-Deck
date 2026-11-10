# First task after takeover

Update after the owner's 0.7.2 request: the local release and validation are
complete; read `RELEASE_072.md` before acting on the original task below. The
owner clarified native firmware alarms work and confirmed Clock alarm sound at
70% plus return to radio after the CGI permissions fix. Do not reopen that
specific quiet-native-alarm report. A full 0.7.2 installation or publication
requires its own authorization. Outstanding hardware checks are listed in the
release evidence, including physical swipe, performance and multi-scene alarms.

Reply in Czech. Read AGENTS.md and CODEX_HANDOFF.md. Treat 0.7.1 as an unconfirmed-on-device baseline, not as a proven fix. Do not rewrite the application.

First establish a reproducible baseline build using the included SDK/vendor archive and run the relevant local tests. Compare any existing Git checkout before copying files. Report the actual commit/working-tree state if Git exists. Preserve the supplied release binaries and user data.

Then focus on the two unresolved audio reports: native alarm too quiet, and Clock sounds previously causing repeated brief radio interruptions. Review radio lifecycle, Clock CGI and firmware sound-volume path. Obtain actual device diagnostics when access is available; without access, prepare the minimum read-only commands and continue independent local tests. Do not adjust firmware volume blindly. Record the current firmware version before interpreting behavior.

Check UI navigation does not stop audio, physical swipe, last-track recall, nested folders, named station groups/logos and meter behavior. Measure meter update rate plus CPU/memory/audio continuity on the Deck before tuning refresh further. A host mock cannot establish those values.

Finish with a concise Czech report: changes, tests, remaining limitations. If a new release is requested, use 0.7.2 unless the owner specifies otherwise, rebuild both components as needed, validate restored NAR and refresh source/checksums. Do not publish or install until the task authorizes it.
