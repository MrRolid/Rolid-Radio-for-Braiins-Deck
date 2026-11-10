"""Read-only diagnostics with isolated proc/firmware and a mocked status API."""
from pathlib import Path
import os
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory() as tmp:
    d = Path(tmp)
    proc = d / 'proc' / '123'
    proc.mkdir(parents=True)
    (proc / 'cmdline').write_bytes(b'/usr/bin/madplay\0-q\0-A\0-18\0/private/music/song.mp3\0')
    firmware = d / 'release'
    firmware.write_text("DISTRIB_RELEASE='test-fw'\nPRIVATE_NETWORK='secret-network'\n")
    wget = d / 'wget'
    wget.write_text('#!/bin/sh\nprintf \'%s\\n\' \'{"version":"0.7.1","status":"alarm","alarm_monitor":"local-madplay","music":{"tracks":["private-title"]},"url":"http://private-server"}\'\n')
    wget.chmod(0o700)
    source = (root / 'tools/alarm-diagnostics.sh').read_text()
    source = source.replace('/proc/', str(d / 'proc') + '/').replace('/etc/openwrt_release', str(firmware))
    script = d / 'diagnostics'
    script.write_text(source)
    output = subprocess.check_output(['sh', str(script)], env={**os.environ, 'PATH': str(d) + ':' + os.environ['PATH']}, text=True)
    for expected in ['test-fw', 'attenuation_db=-18', 'status=alarm', 'version=0.7.1', 'alarm_monitor=local-madplay']:
        assert expected in output, expected
    for private in ['secret-network', '/private/', 'private-title', 'private-server']:
        assert private not in output, private
    assert (proc / 'cmdline').read_bytes().endswith(b'/private/music/song.mp3\0')
print('PASS filtered firmware/player/status diagnostics, no personal paths or library output')
