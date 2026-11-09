"""HTTP and real FFmpeg tests. A paced PCM sink replaces the physical speaker."""
import json,os,subprocess,tempfile,time,urllib.request,urllib.error
from pathlib import Path
root=Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory() as tmp:
 d=Path(tmp);sink=d/'sink.py';sink.write_text('#!/usr/bin/python3\nimport sys,time\nwhile True:\n b=sys.stdin.buffer.read(3840)\n if not b:break\n time.sleep(len(b)/192000)\n');sink.chmod(0o700)
 subprocess.run(['ffmpeg','-v','error','-f','lavfi','-i','sine=frequency=440:duration=3','-c:a','libmp3lame',str(d/'tone.mp3')],check=True);payload=(d/'tone.mp3').read_bytes()
 def start():
  p=subprocess.Popen([str(root/'service/target/debug/rolid-radio-service'),'--web'],env={**os.environ,'ROLID_RADIO_DATA_DIR':tmp,'ROLID_RADIO_APLAY':str(sink)},stderr=subprocess.PIPE)
  for _ in range(100):
   try:urllib.request.urlopen('http://127.0.0.1:18765/status',timeout=.1);return p
   except Exception:time.sleep(.03)
  raise AssertionError('start failed')
 p=start();token=''
 def cmd(action,**kw):
  req=urllib.request.Request('http://127.0.0.1:18766/command',data=json.dumps(dict(action=action,**kw)).encode(),headers={'Authorization':'Bearer '+token,'Content-Type':'application/json'})
  with urllib.request.urlopen(req,timeout=4) as r:return json.load(r)
 def status():return json.load(urllib.request.urlopen('http://127.0.0.1:18765/status'))
 def reject(action,**kw):
  try:cmd(action,**kw)
  except urllib.error.HTTPError as e:assert e.code==400;return
  raise AssertionError('accepted invalid action: '+action)
 def upload(name):
  uid=cmd('music_upload_begin',name=name,size=len(payload))['upload']['id']
  reject('music_upload_chunk',id=uid,offset=1,hex='00');reject('music_upload_finish',id=uid)
  for offset in range(0,len(payload),24576):cmd('music_upload_chunk',id=uid,offset=offset,hex=payload[offset:offset+24576].hex())
  cmd('music_upload_finish',id=uid);return uid
 try:
  reject('music_upload_begin',name='x.wav',size=100);reject('music_upload_begin',name='x.mp3',size=129*1024*1024)
  a=upload('../../test.mp3');b=upload('second.mp3');assert len(list((d/'music').glob('*.mp3')))==2
  cmd('music_folder_create',name='Albums',parent='');f=cmd('music_library')['folders'][0]['id']
  cmd('music_folder_create',name='Live',parent=f);child=cmd('music_library')['folders'][1]['id']
  cmd('music_move',id=a,folder=child);reject('music_folder_delete',id=child);reject('music_folder_delete',id=f)
  cmd('music_folder_rename',id=child,name='Concert');assert cmd('music_library')['tracks'][0]['folder']==child
  reject('music_move',id=a,folder='missing')
  cmd('music_playlist_save',name='Reversed',tracks=[b,a]);pl=cmd('music_library')['playlists'][0]['id']
  reject('music_playlist_save',name='Bad',tracks=['missing']);reject('music_playlist_save',name='Double',tracks=[a,a])
  cmd('music_play',playlist=pl);assert status()['music']['id']==b
  cmd('music_next');assert status()['music']['id']==a
  cmd('radio_mode');cmd('music_move',id=a,folder='');cmd('music_folder_delete',id=child);cmd('music_folder_delete',id=f)
  assert 'access' not in status() and 'key' not in str(status())
  cmd('access_set',enabled=False,key='new-test-key-123')
  assert json.load(urllib.request.urlopen('http://127.0.0.1:18766/status'))['api']==7
  token='new-test-key-123';cmd('access_set',enabled=True,key=token)
  try:urllib.request.urlopen('http://127.0.0.1:18766/security');raise AssertionError('key exposed')
  except urllib.error.HTTPError as e:assert e.code==401
  assert cmd('access_get')['access']['key']==token
  cmd('music_edit',id=a,title='<b>Music</b>',artist='Artist',album='Album');assert cmd('music_library')['tracks'][0]['artist']=='Artist';reject('music_delete',id='../state.json')
  cmd('music_play',id=a);time.sleep(.6);assert status()['status']=='playing',status()
  # Simulate a local alarm player process. The monitor must release the sink.
  alarm=subprocess.Popen(['bash','-c',"exec -a madplay /usr/bin/python3 -c 'import time;time.sleep(1)' /tmp/test-alarm.mp3"])
  time.sleep(.2);assert status()['status']=='alarm',status()
  pos_alarm=status()['music']['position'];time.sleep(.3);assert status()['music']['position']==pos_alarm
  alarm.wait();time.sleep(6.3);assert status()['status']=='playing',status()
  pos=cmd('music_pause')['music']['position'];time.sleep(.3);assert status()['music']['position']==pos
  cmd('music_resume');time.sleep(.3);assert status()['music']['position']>pos
  cmd('volume',value=50);time.sleep(.2);assert status()['music']['position']>=pos
  cmd('mute',value=True);pos=status()['music']['position'];time.sleep(.2);cmd('mute',value=False);time.sleep(.3);assert status()['music']['position']>=pos
  reject('music_delete',id=a)
  deadline=time.time()+8
  while time.time()<deadline and status()['music']['id']!=b:time.sleep(.1)
  assert status()['music']['id']==b,'did not advance on EOF'
  deadline=time.time()+6
  while time.time()<deadline and status()['wanted']:time.sleep(.1)
  assert not status()['wanted'],'last track loops despite repeat off'
  cmd('radio_mode');cmd('music_delete',id=a);assert len(cmd('music_library')['tracks'])==1
  cmd('music_options',repeat=True,shuffle=True)
  cmd('music_upload_begin',name='unfinished.mp3',size=100)
  p.terminate();p.wait(timeout=4);p=start()
  assert cmd('access_get')['access']['key']==token
  assert cmd('music_library')['playlists'][0]['tracks']==[b]
  assert len(cmd('music_library')['tracks'])==1 and not list((d/'music').glob('*.part'))
  assert status()['music']['repeat'] and status()['music']['shuffle']
  assert status()['volume']==50
  # A corrupt index must not be silently replaced by an upload.
  p.terminate();p.wait(timeout=4);(d/'music/library.json').write_text('broken');p=start()
  reject('music_upload_begin',name='new.mp3',size=100);assert (d/'music/library.json').read_text()=='broken'
  print('PASS upload, limits, metadata, paths, real decoding, pause/resume, volume/mute continuity, next/end, persistence, interrupted upload and corrupt index protection')
 finally:p.terminate();p.wait(timeout=4)
