#!/bin/sh
# Run on Deck while its native alarm is sounding. No configuration changes.
printf 'Active local sound-player arguments:\n'
for f in /proc/[0-9]*/cmdline; do
 [ -r "$f" ] || continue
 case "$(tr '\000' ' ' < "$f" 2>/dev/null)" in *madplay*) tr '\000' ' ' < "$f"; printf '\n';; esac
done
printf '\nRadio status:\n'
wget -q -O - http://127.0.0.1:18765/status
printf '\n'
