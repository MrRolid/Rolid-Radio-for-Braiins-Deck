#!/bin/sh
# Run on Deck while its native alarm is sounding. Read-only; omit library,
# source URLs, passwords and personal network configuration from the output.
printf 'Firmware:\n'
uname -r
if [ -r /etc/openwrt_release ]; then
 sed -n '/^DISTRIB_ID=/p; /^DISTRIB_RELEASE=/p; /^DISTRIB_REVISION=/p; /^DISTRIB_TARGET=/p' /etc/openwrt_release
fi
printf '\nLocal madplay processes and attenuation (dB):\n'
for f in /proc/[0-9]*/cmdline; do
 [ -r "$f" ] || continue
 tr '\000' '\n' < "$f" 2>/dev/null | awk '
 NR == 1 { n=split($0,a,"/"); if(a[n]!="madplay") exit; print "madplay" }
 pending { if ($0 ~ /^[-+]?[0-9]+([.][0-9]+)?$/) print "attenuation_db=" $0; pending=0 }
 $0 == "-A" || $0 == "--attenuate" { pending=1 }
 /^--attenuate=[-+]?[0-9]+([.][0-9]+)?$/ { print $0 }
 /^-A[-+]?[0-9]+([.][0-9]+)?$/ { print $0 }
 '
done
printf '\nRadio state (filtered):\n'
wget -q -O - http://127.0.0.1:18765/status |
 sed 's/,/\n/g' | sed -n 's/.*"\(status\|version\|alarm_monitor\)":"\([a-zA-Z0-9_.-]*\)".*/\1=\2/p'
