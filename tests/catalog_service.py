"""Exercise the real HTTP service with isolated persistent data, no audio playback."""
import json,os,subprocess,tempfile,time,http.client
from pathlib import Path
binary=Path(__file__).resolve().parents[1]/'service/target/debug/rolid-radio-service'
def request(value=None):
 c=http.client.HTTPConnection('127.0.0.1',18765,timeout=3)
 c.request('GET' if value is None else 'POST','/status' if value is None else '/command',None if value is None else json.dumps(value),{'Authorization':'Rolid-radio-v1'})
 r=c.getresponse();result=r.status,json.loads(r.read());c.close();return result
with tempfile.TemporaryDirectory() as td:
 Path(td,'state.json').write_text(json.dumps({'name':'Old station','url':'https://example.org/live.mp3','volume':35}))
 def start():
  p=subprocess.Popen([str(binary)],env={**os.environ,'ROLID_RADIO_DATA_DIR':td})
  for _ in range(30):
   try:request();return p
   except OSError:time.sleep(.1)
  raise AssertionError('Service did not start')
 p=start()
 try:
  _,s=request();assert s['api']==4 and s['volume']==35 and s['catalog']['revision']==0
  address=s["donation_address"];assert address.startswith("bc1q") and len(address)==42
  revision=0
  for bank,count in [('cz',8),('en',4),('de',4),('es',4),('ja',3)]:
   code,s=request(dict(action='catalog',revision=revision,bank=bank));assert code==200 and len(s['catalog']['stations'])==count
   revision+=1;assert s['catalog']['revision']==revision and s['name']=='Old station' and not s['wanted']
   assert request()[1]['catalog']==s['catalog']
  code,s=request(dict(action='catalog',revision=0,bank='cz',seed=True));assert code==200 and s['catalog']['bank']=='ja'
  assert request(dict(action='catalog',revision=0,bank='cz'))[0]==400
  for url in ['https://example.org/list.m3u','file:///tmp/a.mp3']:
   assert request(dict(action='catalog',revision=revision,bank='custom',stations=[dict(name='Bad',url=url)]))[0]==400
   assert request(dict(action='play',url=url))[0]==400
  custom=[dict(name='Shared custom',url='https://example.org/live.mp3')]
  assert request(dict(action='catalog',revision=revision,bank='custom',stations=custom))[0]==200
  p.terminate();p.wait(5);p=start()
  _,s=request();assert s['catalog']['stations']==custom and s['catalog']['revision']==6 and s['volume']==35 and not s['wanted']
  assert s["donation_address"]==address
  print('PASS 23 bundled stations, shared clients, stale revision/seed protection, invalid formats, persistence and preference migration')
 finally:p.terminate();p.wait(5)
