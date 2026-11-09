import datetime as dt
import json
import struct as s
from pathlib import Path
from zoneinfo import ZoneInfo
import wasmtime as w

engine=w.Engine()
module=w.Module.from_file(engine,str(Path(__file__).parents[1]/'package/root/lib/bmc-widgets/rolid-radio/lib/wasm/rolid-radio.wasm'))
def value(v):
    if v is None:return 4,b''
    if isinstance(v,bool):return 3,s.pack('<?',v)
    if isinstance(v,int):return 1,s.pack('<i',v)
    if isinstance(v,list):return 5,s.pack('<I',len(v))+b''.join(bytes([k])+b for k,b in map(value,v))
    if isinstance(v,dict):return 6,packed(v)
    b=v.encode();return 0,s.pack('<I',len(b))+b

def packed(values):
    out=s.pack('<I',len(values))
    for key,v in values.items():
        k,b=value(v);key=key.encode();out+=s.pack('<BH',k,len(key))+key+b
    return out

def timestamp(text):return int(dt.datetime.fromisoformat(text).timestamp())

class Device:
    def __init__(self,width=1280,height=480,params=None,kv=None,now=None,system_alarm=None):
        self.width,self.height=width,height;self.params={"use_audio_service":False,**(params or {})};self.pv=1;self.sv=1
        self.now=now if now is not None else timestamp('2026-11-01T12:59:59+01:00')
        self.kv=kv if kv is not None else {};self.tree=b'';self.leds=[];self.led_stops=0;self.sounds=[];self.stops=[];self.audio={};self.delay=0;self.requests=0;self.click=None;self.timer=0;self.timer_count=0;self.fetches=[];self.paths=[];self.pending=set();self.fetch_delays=[];self.night=False;self.system_alarm=system_alarm
        self.store=w.Store(engine);linker=w.Linker(engine)
        for i in module.imports:linker.define_func(i.module,i.name,i.type,self.callback(i.name),access_caller=True)
        self.instance=linker.instantiate(self.store,module);self.e=self.instance.exports(self.store)
        assert self.call('__bmc_sdk_init')==9<<16
        self.call('init')
    def call(self,name,*args):return self.e[name](self.store,*args)
    def render(self):self.call('render',1000);return self.tree
    def tick(self,seconds=1):self.now+=seconds;self.call('__on_timer',self.timer,seconds)
    def change(self,**values):self.params.update(values);self.pv+=1;self.call('on_params_update')
    def tap(self,key):self.click=key;self.call('on_touch');self.render();self.render()
    def callback(self,name):
        def call(c,*a):
            mem=c.get('memory')
            read=lambda p,n:bytes(mem.read(c,p,p+n))
            if name in ['host_fetch','host_fetch_after']:
                delay=0
                if name=='host_fetch_after':delay=a[0];a=a[1:]
                self.fetch_delays.append(delay)
                timeout,mp,ml,up,ul,hp,hl,bp,bl=a
                method,url,headers,body=read(mp,ml),read(up,ul),read(hp,hl),read(bp,bl)
                assert url in [b'http://127.0.0.1:18765/command',b'http://127.0.0.1:18765/status',b'http://127.0.0.1:18765/levels']
                self.fetches.append((method.decode(),json.loads(body) if body else None));self.paths.append(url.decode());self.pending.add(len(self.fetches));return len(self.fetches)
            if name=='host_register_bitmap':return 2
            if name=='host_register_svg_package':
                assert read(a[2],9)==b"BMCREFV1\x01";asset_id=read(a[2]+9,32).hex()
                asset=Path(__file__).parents[1]/'package/root/lib/bmc-widgets/rolid-radio/lib/assets/v1/svg'/f'{asset_id}.asset'
                assert asset.is_file() and asset.stat().st_size>0,asset
                return 1
            if name=='host_params_version':return self.pv
            if name=='host_system_version':return self.sv
            if name.endswith('_snapshot'):
                if name=='host_params_snapshot':b=packed(self.params)
                else:
                    zone=b'Europe/Prague';b=s.pack('<IBH',2+(self.system_alarm is not None),0,len(zone))+zone+bytes([8,self.night])
                    if self.system_alarm:
                        at,label=self.system_alarm;label=label.encode();b+=s.pack('<BBqH',7,1,at*1000,len(label))+label
                if a[1]>=len(b):mem.write(c,b,a[0])
                return len(b)
            if name=='host_widget_size':return (self.width<<32)|self.height
            if name=='host_widget_viewport_shape':return 0
            if name=='host_get_system_time':mem.write(c,s.pack('<q',self.now),a[0]);return
            if name=='host_submit_tree':
                p,n,ww,hh=a;assert (ww,hh)==(self.width,self.height);self.tree=read(p,n);return
            if name=='host_request_frame_after':self.delay=a[0];return
            if name=='host_request_frame':self.requests+=1;return
            if name=='host_get_touch_click':
                key=read(a[0],a[1]).decode()
                if key==self.click:
                    self.click=None;mem.write(c,s.pack('<ffff',10,10,100,40),a[2]);return 1
                return 0
            if name=='host_get_touch_drag':return 0
            if name=='host_timer_start':
                if a[0]<1000:return 0
                self.timer_count+=1
                if self.timer_count==1:self.timer=1
                return self.timer_count
            if name=='host_timer_stop':return
            if name=='host_register_audio':
                b=read(a[0],a[1]);tag=read(a[2],a[3]).decode();assert b[:4]==b'RIFF';key=len(self.audio)+1;self.audio[key]=tag;return key
            if name=='host_audio_play':assert a[0] in self.audio;self.sounds.append((self.now,self.audio[a[0]],a[1]));return
            if name=='host_led_set_temporary':self.leds.append(a);return
            if name=='host_led_stop':self.led_stops+=1;return
            if name=='host_audio_stop':self.stops.append(a[0]);return
            if name=='host_kv_set':self.kv[read(a[0],a[1]).decode()]=read(a[2],a[3]);return
            if name=='host_kv_get':
                b=self.kv.get(read(a[0],a[1]).decode())
                if b is None:return -1
                if a[3]>=len(b):mem.write(c,b,a[2])
                return len(b)
            if name=='host_log':return
            raise AssertionError(name)
        return call


def reply(d,index=1,**kw):
 d.pending.discard(index)
 kw.setdefault('api',3);kw.setdefault('donation_address','bc1qnpk42l7x6pexp0fw4a5j2tm3l5uerc6drpyw3l');kw.setdefault('catalog',{'revision':1,'bank':'custom','stations':d.params.get('stations',stations)})
 data=json.dumps(dict(status='stopped',volume=60,muted=False,wanted=False,name='Radiozurnal',url='https://example.com/0.mp3') | kw).encode()
 ptr=d.call('__alloc',len(data));d.e['memory'].write(d.store,data,ptr);d.call('__on_fetch_response',index,200,ptr,len(data))
stations=[{'name':f'Station {i+1}','url':f'https://example.com/{i}.mp3'} for i in range(16)]
def current(d,**kw):
 pending=[i for i in sorted(d.pending) if not d.paths[i-1].endswith('/levels')]
 if not pending:d.tick();pending=[i for i in sorted(d.pending) if not d.paths[i-1].endswith('/levels')]
 reply(d,pending[0],**kw)
def action(d,key,**kw):
 d.tap(key);current(d,**kw)
 while any(d.fetches[i-1][0]=='POST' for i in d.pending):current(d,**kw)
 d.render()
for lang in ['cz','en','de','es','ja']:
 for width,height in [(317,238),(638,238),(638,480),(1280,480)]:
  d=Device(width,height,params={'language':lang,'stations':stations});assert d.timer==1
  current(d);d.render();count=12 if width>=960 else 8 if width>=600 else 5
  for i in range(count):assert f'p{i}'.encode() in d.tree
  for key in ['open_list','open_settings','toggle','up','down','mute']:assert key.encode() in d.tree
  if width>=600:assert b'prev' in d.tree and b'next' in d.tree
  assert 100 in d.fetch_delays
  Path(f'/tmp/radio-{lang}-{width}-{height}.tree').write_bytes(d.tree)
  action(d,'p4');assert any(x[1] and x[1].get('url')==stations[4]['url'] for x in d.fetches)
  action(d,'up');assert any(x[1]=={'action':'volume','value':65} for x in d.fetches)
  d.tap('open_list');d.tap('page_prev');assert b'station0' in d.tree
  if lang=='cz':Path(f'/tmp/radio-list-{width}-{height}.tree').write_bytes(d.tree)
  d.tap('page_next');assert b'station5' in d.tree
  d.tap('list_settings');assert b'rgb_toggle' in d.tree
  if lang=='cz':Path(f'/tmp/radio-settings-{width}-{height}.tree').write_bytes(d.tree)
  d.tap('banks');assert b'bank_5' in d.tree
  if lang=='cz':Path(f'/tmp/radio-menu-{width}-{height}.tree').write_bytes(d.tree)
 print('PASS',lang,'5/8/12 presets + lists + settings + arrows + delayed polling')
d=Device(317,238,params={'stations':stations});current(d);d.render();assert b'PLAY' in d.tree # English default
current(d,status='retrying',wanted=True);d.render();assert b'RECONNECTING' in d.tree
for _ in range(8):current(d,status='playing',wanted=True)
d.render();assert b'PLAYING' in d.tree and b'RECONNECTING' not in d.tree
assert all(x in (0,100) for x in d.fetch_delays)
before=len(d.fetches);d.tick();assert len(d.fetches)==before+1
for width,height in [(317,238),(638,238),(638,480),(1280,480)]:
 for i,address in enumerate(['bc1qnpk42l7x6pexp0fw4a5j2tm3l5uerc6drpyw3l','bc1q2hadjw5nwtyywhvd50ll0qelglgjeqn37988r9','bc1qg24xphhjk05l7xaqfzr4w9nhgje39jm46xfjhv','bc1qdyyp3p4wdde2nrtlqgjw84272nu6436h9yydat']):
  d=Device(width,height,params={'stations':stations});current(d,donation_address=address);d.tap('open_settings');d.tap('donate');assert b'donate_back' in d.tree
  Path(f'/tmp/radio-donation{i}-{width}-{height}.tree').write_bytes(d.tree)
  d.tap('donate_back');assert b'rgb_toggle' in d.tree
  d.tap('info');assert b'info_back' in d.tree;d.tap('info_back')
d=Device(317,238,params={'stations':stations});current(d);d.tap('open_settings');d.tap('rgb_toggle');d.tap('rgb_level');d.tap('lang')
assert json.loads(d.kv['panel-settings-v3'])=={'rgb':False,'max':20,'lang':'cz'}
e=Device(317,238,params={'stations':stations},kv=d.kv);current(e);e.tap('open_settings');assert b'RGB OFF' in e.tree and b'RGB 20%' in e.tree
e.change(rgb_notify=False,rgb_max=10,language='cz');e.render();assert b'RGB OFF' in e.tree and b'RGB 10%' in e.tree
action(e,'sleep');assert any(x[1]=={'action':'sleep','minutes':15} for x in e.fetches)
d=Device(params={'stations':stations,'rgb_notify':True,'rgb_max':15});current(d,remote_revision=7,status='playing',meter={'rms':[500,700],'peak':[800,900]});d.render();assert not d.leds
current(d,remote_revision=8);assert len(d.leds)==1 and d.leds[-1][-1]==0 and d.leds[-1][-2]==900
for _ in range(5):d.render();current(d,remote_revision=8)
assert len(d.leds)==1
current(d,remote_revision=0);assert len(d.leds)==1
current(d,remote_revision=1);assert len(d.leds)==2
d.change(rgb_notify=False);current(d,remote_revision=2);assert len(d.leds)==2
# Explicit draft publishing only; other parameter changes never send a catalog.
d=Device(params={'stations':stations});current(d);d.change(language='de');current(d)
assert not any(x[1] and x[1].get('edit') for x in d.fetches)
d.change(publish_stations=1);current(d)
assert any(x[1] and x[1].get('edit') and x[1]['stations']==stations for x in d.fetches)
d=Device(params={'stations':stations,'show_donation':True});current(d);d.render();assert b'donate_back' in d.tree
print('PASS remote-only RGB, no repeated pulse, restart baseline, explicit station publishing and native BTC preview')
# Music layouts and command routing use the actual WASM widget.
for width,height in [(317,238),(638,238),(638,480),(1280,480)]:
 d=Device(width,height,params={'stations':stations})
 music={'id':'a'*32,'position':61.5,'repeat':False,'shuffle':False,'tracks':[{'id':c*32,'title':'Track '+c} for c in 'abcde']}
 current(d,mode='music',music=music,name='Local track',status='playing',wanted=True);d.tap('radio_music');d.render()
 assert b'music_toggle' in d.tree and b'music_title' in d.tree
 Path(f'/tmp/radio-music-{width}-{height}.tree').write_bytes(d.tree)
 action(d,'music_toggle');assert any(f[1]=={'action':'music_pause'} for f in d.fetches)
 current(d,mode='music',music=music,name='Local track',status='paused',wanted=True)
 d.tap('music_library');assert b'music_item0' in d.tree
 Path(f'/tmp/radio-musiclist-{width}-{height}.tree').write_bytes(d.tree)
 action(d,'music_item1');assert any(f[1]=={'action':'music_play','id':'b'*32} for f in d.fetches)
 current(d,mode='music',music=music,name='Local track',status='playing',wanted=True)
 d.tap('music_radio');assert not any(f[1]=={'action':'radio_mode'} for f in d.fetches);assert b'radio_music' in d.tree
print('PASS music UI, library paging and pause/play/radio commands in four sizes')
# Direct Music entry and group routing on the actual WASM code.
for width,height in [(317,238),(638,238),(638,480),(1280,480)]:
 d=Device(width,height,params={'stations':stations});current(d);d.tap('radio_music');assert b'music_toggle' in d.tree
 music={'id':'a'*32,'position':0,'tracks':[{'id':'a'*32,'title':'Alpha','folder':'folder'}], 'groups':{'folders':[{'id':'folder','name':'Album','parent':''}],'playlists':[{'id':'pl','name':'Favorites','tracks':['a'*32]}]}}
 current(d,mode='music',music=music,wanted=True,status='playing');d.render()
 if width<600:d.tap('music_library')
 d.tap('music_groups');assert b'library:groups' in d.tree
 # Root, All, Folder, Playlist are paged in small/medium.
 d.tap('music_item3')
 assert b'Alpha' in d.tree
 d.tap('music_item0');current(d,mode='music',music=music)
 assert any(f[1]=={'action':'music_play','id':'a'*32,'playlist':'pl'} for f in d.fetches)
d=Device(params={'stations':stations});current(d);d.change(web_key='replacement-key',web_key_required=False,apply_access=1);current(d)
assert any(f[1]=={'action':'access_set','key':'replacement-key','enabled':False} for f in d.fetches)
print('PASS direct MUSIC entry, playlist browser commands and explicit native access settings')
# Native group selection, nested folders and Music recall preserve playback.
d=Device(params={'stations':stations});current(d,bank_names=['Jazz','Rock']);d.tap('catalog');assert b'Jazz' in d.tree and b'Rock' in d.tree
action(d,'bank_6');assert any(f[1] and f[1].get('bank')=='Jazz' for f in d.fetches)
d=Device(params={'stations':stations});music={'last_id':'a'*32,'id':None,'tracks':[{'id':'a'*32,'title':'Nested song','folder':'artist'}],'groups':{'folders':[{'id':'genre','parent':'','name':'Jazz'},{'id':'artist','parent':'genre','name':'Artist'}],'playlists':[]}}
current(d,mode='radio',music=music,wanted=True,status='playing');d.tap('radio_music');action(d,'music_toggle');assert any(f[1]=={'action':'music_resume'} for f in d.fetches)
current(d,mode='music',music=music,wanted=True,status='playing');d.tap('music_library');d.tap('music_groups');d.tap('music_item2');assert b'DIR / Artist' in d.tree;d.tap('music_item0');assert b'Nested song' in d.tree
print('PASS dynamic groups, nested native folder navigation and remembered-track Play')
