#!/bin/sh
set -eu
cd "$(dirname "$0")"
[ "$(id -u)" = 0 ] || { echo 'Run this installer as root.'; exit 1; }
[ "$(uname -m)" = armv7l ] || { echo 'This package requires an ARMv7 Braiins Deck.'; exit 1; }
for p in /usr/bin/madplay /usr/bin/wget /etc/rc.common /lib/functions/procd.sh; do
 [ -r "$p" ] || { echo "Required file not found: $p"; exit 1; }
done
[ -d /mnt/data ]
[ -s rolid-radio-service ]
root=/mnt/data/rolid-radio
if [ -e "$root" ] && [ ! -f "$root/.rolid-radio-v1" ]; then echo 'Refusing to overwrite an unrecognized data directory.'; exit 1; fi
if [ -e /etc/init.d/rolid-radio ]; then
 grep -q ROLID_RADIO_SERVICE_V1 /etc/init.d/rolid-radio || exit 1
 /etc/init.d/rolid-radio stop || :
fi
umask 077
mkdir -p "$root"
if [ -f "$root/rolid-radio-service" ]; then cp "$root/rolid-radio-service" "$root/rolid-radio-service.previous"; fi
cp rolid-radio-service "$root/rolid-radio-service.new"
chmod 700 "$root/rolid-radio-service.new"
mv "$root/rolid-radio-service.new" "$root/rolid-radio-service"
: > "$root/.rolid-radio-v1"
cp rolid-radio.init /etc/init.d/rolid-radio
chmod 755 /etc/init.d/rolid-radio
/etc/init.d/rolid-radio enable
/etc/init.d/rolid-radio start
sleep 2
wget -O - http://127.0.0.1:18765/status
printf '\nRoliD Radio installation complete.\n'

printf '\nWeb: http://<Deck-IP>:18766\nAccess key: '
cat "$root/web-token"
printf '\n'
