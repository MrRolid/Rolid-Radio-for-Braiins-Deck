"""0.7.1 regressions: password default, groups, levels, network MP3 and last track."""
import subprocess,tempfile,time,urllib.request,urllib.error,json,os
from pathlib import Path
r=Path(__file__).resolve().parents[1]
def get(path='/status',data=None,auth='',port=18765):
 req=urllib.request.Request(f'http://127.0.0.1:{port}'+path,data=json.dumps(data).encode() if data is not None else None,headers={'Authorization':auth or 'Rolid-radio-v1','Content-Type':'application/json'})
 return json.load(urllib.request.urlopen(req,timeout=4))
def cmd(action,**kw):return get('/command',dict(action=action,**kw))
with tempfile.TemporaryDirectory() as tmp:
 d=Path(tmp);(d/'web-token').write_text('a'*64)
 sink=d/'sink';sink.write_text('#!/usr/bin/python3\nimport sys,time\nwhile sys.stdin.buffer.read(3840):time.sleep(.02)\n');sink.chmod(0o700)
 server=subprocess.Popen(['python3',str(r/'demo-network/serve.py')],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 p=subprocess.Popen([str(r/'service/target/debug/rolid-radio-service'),'--web'],env={**os.environ,'ROLID_RADIO_DATA_DIR':tmp,'ROLID_RADIO_APLAY':str(sink)},stderr=subprocess.PIPE)
 try:
  for _ in range(100):
   try:s=get();break
   except Exception:time.sleep(.03)
  assert get(port=18766)['access_enabled']==False
  assert cmd('access_get')['access']['key']==''
  cmd('access_set',enabled=True,key='my password');assert get(auth='Bearer my password',port=18766)['api']==7
  cmd('access_set',enabled=True,key='');assert not get(port=18766)['access_enabled']
  st=[{'name':'Jazz','url':'https://example.com/jazz.mp3'}]
  s=cmd('catalog',bank='Jazz',edit=True,revision=0,stations=st);assert 'Jazz' in s['bank_names']
  s=cmd('catalog',bank='Evening Jazz',rename_from='Jazz',edit=True,revision=1,stations=st);assert 'Jazz' not in s['bank_names'] and 'Evening Jazz' in s['bank_names']
  assert set(get('/levels'))=={'meter'}
  start=time.monotonic()
  for _ in range(50):get('/levels')
  assert time.monotonic()-start<3
  cmd('music_folder_create',name='Jazz',parent='');f=cmd('music_library')['folders'][0]['id']
  cmd('music_folder_create',name='Demo artist',parent=f);f=cmd('music_library')['folders'][1]['id']
  cmd('music_network_add',name='Network demo',url='http://127.0.0.1:18767/demo.mp3',folder=f)
  track=cmd('music_library')['tracks'][0];assert track['bytes']==0
  cmd('music_play',id=track['id']);time.sleep(.7);assert get()['status']=='playing',get()
  assert get('/levels')['meter'] is not None
  cmd('radio_mode');cmd('music_resume');time.sleep(.4);assert get()['music']['id']==track['id'] and get()['status']=='playing'
  # Explicit reservation must not be shortened by the process monitor.
  alarm=subprocess.Popen(['bash','-c',"exec -a madplay /usr/bin/python3 -c 'import time;time.sleep(.3)' /tmp/test-alarm.mp3"])
  time.sleep(.1);cmd('duck');alarm.wait();time.sleep(6.3);assert get()['status']=='alarm',get()
  time.sleep(1.8);assert get()['status']=='playing',get()
  assert not list((d/'music').glob('*.mp3')),'Network track was downloaded permanently'
  print('PASS 0.7.1 default empty password, legacy token ignored, custom password, group rename, lightweight meters, nested folders, real HTTP MP3 playback, last-track recall, alarm hold')
 finally:p.terminate();p.wait(timeout=4);server.terminate();server.wait(timeout=4)
