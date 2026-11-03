"""Real HTTP tests: authenticated web, shared banks, sleep and persistence."""
import subprocess,tempfile,time,urllib.request,urllib.error,json,os
from pathlib import Path
binary=Path(__file__).resolve().parents[1]/'service/target/debug/rolid-radio-service'
def http(path='/status',data=None,token=None,port=18765):
 h={'Content-Type':'application/json'}
 if token:h['Authorization']='Bearer '+token
 elif data is not None:h['Authorization']='Rolid-radio-v1'
 req=urllib.request.Request(f'http://127.0.0.1:{port}{path}',data=json.dumps(data).encode() if data is not None else None,headers=h)
 with urllib.request.urlopen(req,timeout=3) as r:return r.status,r.read()
def get(**kw):return json.loads(http(**kw)[1])
def cmd(**data):return get(path='/command',data=data)
with tempfile.TemporaryDirectory() as tmp:
 def start():
  p=subprocess.Popen([str(binary),'--web'],env={**os.environ,'ROLID_RADIO_DATA_DIR':tmp},stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
  for _ in range(100):
   try:http();return p
   except Exception:time.sleep(.02)
  raise AssertionError('service did not start')
 p=start()
 try:
  token=(Path(tmp)/'web-token').read_text();assert len(token)==64
  initial=get();assert initial['api']==4;address=initial['donation_address']
  assert http(path='/',port=18766)[0]==200
  try:http(port=18766);raise AssertionError('unauthenticated access')
  except urllib.error.HTTPError as e:assert e.code==401
  assert get(token=token,port=18766)['donation_address']==address
  qr=get(path='/donation',token=token,port=18766);assert qr['address']==address and len(qr['matrix'])==41
  assert len(cmd(action='catalog',bank='cz',revision=0)['catalog']['stations'])==8
  stations=[{'name':'AAC <test>','url':'https://rozhlas.stream/plus_high.aac'},{'name':'MP3','url':'https://rozhlas.stream/plus.mp3'}]
  get(path='/command',port=18766,token=token,data={'action':'catalog','edit':True,'bank':'cz','revision':1,'stations':stations})
  assert get()['catalog']['stations']==stations
  assert get()['remote_revision']==1
  for volume in [65,65]:get(path='/command',port=18766,token=token,data={'action':'volume','value':volume})
  assert get()['remote_revision']==2
  network=get();assert (network['panel_url'] is None and network['panel_interface'] is None) or (network['panel_url'].startswith('http://') and network['panel_interface'])
  assert get(path='/command',port=18766,token=token,data={'action':'rgb_test'})['remote_revision']==3
  cmd(action='catalog',bank='en',revision=2)
  assert cmd(action='catalog',bank='cz',revision=3)['catalog']['stations']==stations
  assert cmd(action='catalog',bank='custom',revision=4)['catalog']['stations']==stations
  try:cmd(action='catalog',edit=True,bank='cz',revision=1,stations=stations);raise AssertionError('stale update')
  except urllib.error.HTTPError as e:assert e.code==400
  assert cmd(action='sleep',minutes=15)['sleep_remaining'] in (899,900)
  assert cmd(action='sleep',minutes=0)['sleep_remaining'] is None
  begin=time.monotonic()
  for _ in range(30):assert get()['catalog']['stations']==stations
  elapsed=time.monotonic()-begin;assert elapsed<2,elapsed
  print('PASS 30 local status requests:',round(elapsed,3),'s; auth, editable banks, revisions, sleep')
  p.terminate();p.wait(timeout=4);p=start()
  assert get()['catalog']['stations']==stations and get()['donation_address']==address
  assert (Path(tmp)/'web-token').read_text()==token
  print('PASS shared list + donation + web key survive restart')
 finally:p.terminate();p.wait(timeout=4)
