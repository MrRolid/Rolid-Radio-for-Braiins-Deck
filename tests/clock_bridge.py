"""Exercise the installed CGI logic with isolated paths and mock audio binaries."""
from pathlib import Path
import tempfile,subprocess,os,time
source=(Path(__file__).resolve().parents[1]/'service-package/clock-audio-bridge').read_text()
with tempfile.TemporaryDirectory() as tmp:
 d=Path(tmp);root=d/'audio';(root/'sounds').mkdir(parents=True);(root/'sounds/gong.mp3').write_bytes(b'test');(root/'sounds/alarm.mp3').write_bytes(b'test')
 player=d/'madplay';player.write_text('#!/usr/bin/python3\nimport time\nfrom pathlib import Path\np=Path('+repr(str(d/'events'))+')\nwith p.open("a") as f:f.write("play\\n")\ntime.sleep(4)\n');player.chmod(0o700)
 radio=d/'radio';radio.write_text('#!/bin/sh\necho duck >> '+str(d/'events')+'\n');radio.chmod(0o700)
 (root/'stop.sh').write_text('#!/bin/sh\nif [ -f '+str(d/'state/player.pid')+' ]; then read -r p < '+str(d/'state/player.pid')+'; kill "$p" 2>/dev/null || :; rm -f '+str(d/'state/player.pid')+'; fi\n')
 script=d/'cgi';script.write_text(source.replace('/mnt/data/rolid-audio',str(root)).replace('/tmp/rolid-audio',str(d/'state')).replace('/usr/bin/madplay',str(player)).replace('/mnt/data/rolid-radio/rolid-radio-service',str(radio)).replace('/proc/',str(d/'proc')+'/'))
 def request(query):return subprocess.check_output(['sh',str(script)],env={**os.environ,'REMOTE_ADDR':'127.0.0.1','REQUEST_METHOD':'POST','HTTP_AUTHORIZATION':'Rolid-local-v1','QUERY_STRING':query}).decode()
 try:
  assert 'accepted' in request('action=gong&volume=70');time.sleep(.15)
  proc=d/'proc'/(d/'state/player.pid').read_text().strip();proc.mkdir(parents=True);(proc/'cmdline').write_bytes(b'madplay\0'+str(root/'sounds/gong.mp3').encode()+b'\0')
  assert 'playing' in request('action=gong&volume=70')
  assert (d/'events').read_text().splitlines()==['duck','play']
  assert 'stopped' in request('action=stop')
  print('PASS Clock CGI reserves speaker before playback, duplicate request does not restart sound, explicit stop')
 finally:request('action=stop')
