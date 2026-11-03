mod donation;mod i18n;mod jp;mod design;
use bmc_wasm_sdk::*;
use std::{cell::RefCell,time::Duration};
use serde_json::{Value,json};
struct State {stations:Vec<(String,String)>,selected:usize,status:Value,message:String,busy:bool,lang:String,volume:u64,muted:bool,draft:Vec<(String,String)>,menu:bool,list:bool,settings:bool,info:bool,page:usize,seeded:bool,donate:bool,rgb:bool,rgb_max:u32,remote_revision:Option<u64>}
impl Default for State {fn default()->Self {Self{stations:vec![],selected:0,status:json!({}),message:String::new(),busy:false,lang:"en".into(),volume:60,muted:false,draft:vec![],menu:false,list:false,settings:false,info:false,page:0,seeded:false,donate:false,rgb:false,rgb_max:15,remote_revision:None}}}
thread_local! {static QUEUE:RefCell<std::collections::VecDeque<Value>>=RefCell::new(std::collections::VecDeque::new());static STATE:RefCell<State>=RefCell::new(State::default());static TIMER:RefCell<Option<timer::Timer>>=const{RefCell::new(None)};}
fn configure(){STATE.with_borrow_mut(|s|{
 s.rgb=params::current().get_bool("rgb_notify").unwrap_or(true);s.rgb_max=params::current().get_i32("rgb_max").unwrap_or(15).clamp(0,30) as u32;
 s.donate=params::current().get_bool("show_donation").unwrap_or(false);if !s.rgb{led::stop();}
 s.lang=params::current().get_str("language").unwrap_or("en").to_string();s.draft.clear();
 if let Some(items)=params::current().get_list("stations"){for item in items.iter().take(32){if let Some(o)=item.as_object(){if let (Some(n),Some(u))=(o.get("name").and_then(|x|x.as_str()),o.get("url").and_then(|x|x.as_str())){if !n.trim().is_empty()&&!u.trim().is_empty(){s.draft.push((n.into(),u.into()));}}}}}
 if s.stations.is_empty(){s.stations=if s.draft.is_empty(){builtin(&s.lang)}else{s.draft.clone()};s.selected=0;}
 });}
fn builtin(lang:&str)->Vec<(String,String)>{let v:Value=serde_json::from_str(include_str!("../stations.json")).unwrap();parse_stations(v.get(lang).unwrap_or(&v["cz"]))}
fn parse_stations(v:&Value)->Vec<(String,String)>{v.as_array().map(|a|a.iter().filter_map(|s|Some((s["name"].as_str()?.into(),s["url"].as_str()?.into()))).collect()).unwrap_or_default()}
fn catalog_command(s:&State,bank:&str,seed:bool)->Value{json!({"action":"catalog","bank":bank,"seed":seed,"revision":s.status["catalog"]["revision"].as_u64().unwrap_or(0),"stations":if seed{s.draft.iter().map(|(n,u)|json!({"name":n,"url":u})).collect::<Vec<_>>()}else{vec![]}})}
fn response(r:&net::FetchResponse){let seed=STATE.with_borrow_mut(|s|{
 s.busy=false;
 if r.ok(){if let Ok(v)=serde_json::from_slice::<Value>(r.body()){if v["status"].is_string(){
  if QUEUE.with_borrow(|q|q.is_empty()){s.volume=v["volume"].as_u64().unwrap_or(60);s.muted=v["muted"].as_bool().unwrap_or(false);}
  if let Some(n)=v["remote_revision"].as_u64(){if s.remote_revision.is_some_and(|old|n>old)&&s.rgb&&s.rgb_max>0{let gain=s.rgb_max as f32/100.0;led::set_effect(led::LedEffect::Solid,Color::from_hex(((255.0*gain) as u32)<<16|((112.0*gain) as u32)<<8),1000,Some(900));}s.remote_revision=Some(n);}
  let stations=parse_stations(&v["catalog"]["stations"]);
  if !stations.is_empty(){let changed=s.status["catalog"]["revision"]!=v["catalog"]["revision"]||s.status["url"]!=v["url"];s.stations=stations;if changed{s.selected=s.stations.iter().position(|x|Some(x.1.as_str())==v["url"].as_str()).unwrap_or(0);}s.selected=s.selected.min(s.stations.len()-1);}
  let capable=v["api"].as_u64().unwrap_or(0)>=3;s.status=v;s.message=if capable{String::new()}else{"error".into()};
  if capable&&!s.seeded&&s.status["catalog"]["revision"].as_u64()==Some(0){s.seeded=true;
    let legacy=s.draft.len()==1&&s.draft[0].1=="https://rozhlas.stream/radiozurnal_mp3_128.mp3";
    let bank=if s.draft.is_empty()||legacy||s.draft==builtin(&s.lang){s.lang.as_str()}else{"custom"};return Some(catalog_command(s,bank,true));
  }return None;
 }}}
 s.message="error".into();None
 });request_frame();if let Some(c)=seed{send(Some(c));}else if QUEUE.with_borrow(|q|!q.is_empty()){send(None);}else{schedule();}}
fn send(command:Option<Value>){if let Some(c)=command {QUEUE.with_borrow_mut(|q|{if q.len()<16 {q.push_back(c);}});}let ready=STATE.with_borrow_mut(|s|{if s.busy{false}else{s.busy=true;true}});if !ready{return}let command=QUEUE.with_borrow_mut(|q|q.pop_front());let body=command.as_ref().map(Value::to_string);let req=if let Some(b)=&body{net::FetchRequest::post("http://127.0.0.1:18765/command").headers("Authorization: Rolid-radio-v1\nContent-Type: application/json").body(b.as_bytes())}else{net::FetchRequest::get("http://127.0.0.1:18765/status")};if req.timeout(Duration::from_secs(3)).send(response).is_none(){STATE.with_borrow_mut(|s|{s.busy=false;s.message="error".into();});}}
fn schedule(){let delay=STATE.with_borrow_mut(|s|{s.busy=true;if s.message.is_empty(){150}else{1500}});if net::FetchRequest::get("http://127.0.0.1:18765/status").timeout(Duration::from_secs(3)).send_after(delay,response).is_none(){STATE.with_borrow_mut(|s|s.busy=false);}}
#[unsafe(no_mangle)] pub extern "C" fn init(){configure();STATE.with_borrow_mut(|s|{if let Some(v)=kv::get("panel-settings").and_then(|b|serde_json::from_slice::<Value>(&b).ok()){s.rgb_max=v["max"].as_u64().unwrap_or(s.rgb_max as u64).min(30) as u32;s.lang=v["lang"].as_str().unwrap_or(&s.lang).to_owned();}if let Some(v)=kv::get("panel-settings-v3").and_then(|b|serde_json::from_slice::<Value>(&b).ok()){s.rgb=v["rgb"].as_bool().unwrap_or(s.rgb);s.rgb_max=v["max"].as_u64().unwrap_or(s.rgb_max as u64).min(30) as u32;s.lang=v["lang"].as_str().unwrap_or(&s.lang).to_owned();}});TIMER.set(timer::start(1000,|_,_|send(None)));send(None);request_frame();}
#[unsafe(no_mangle)] pub extern "C" fn on_params_update(){kv::set("panel-settings",b"");kv::set("panel-settings-v3",b"");configure();let changed=params::current().get_i32("publish_stations").unwrap_or(0)!=params::previous().get_i32("publish_stations").unwrap_or(0);if changed{let c=STATE.with_borrow(|s|{s.status["catalog"]["revision"].as_u64().filter(|_|!s.draft.is_empty()).map(|rev|json!({"action":"catalog","bank":"custom","edit":true,"revision":rev,"stations":s.draft.iter().map(|(n,u)|json!({"name":n,"url":u})).collect::<Vec<_>>()}))});if let Some(c)=c{send(Some(c));}}request_frame();}
#[unsafe(no_mangle)] pub extern "C" fn on_touch(){request_frame();}

fn play(s:&State)->Value {json!({"action":"play","name":s.stations[s.selected].0,"url":s.stations[s.selected].1})}
#[unsafe(no_mangle)] pub extern "C" fn render(_:u32){
 let vp=widget_viewport();let commands=STATE.with_borrow_mut(|s|{
 use design::*;use i18n::tr;
 let l=layout(vp.width,vp.height);let w=l.width;let gap=4.0;
 let measured=s.status["meter"].is_object()&&s.status["status"]=="playing"&&s.message.is_empty();
 if s.donate{
  let address=s.status["donation_address"].as_str().unwrap_or("");let idx=donation::ADDRESSES.iter().position(|a|*a==address);
  let area_h=vp.height as f32-68.0;let left=(w*0.56).floor();let right=w-left-4.0;
  let mut qr=vec![];
  if let Some(i)=idx{let rows=donation::MATRICES[i];let n=rows.len() as f32;let px=(left.min(area_h)/n).floor().max(1.0);let side=n*px;let x=((left-side)/2.0).floor();let y=((area_h-side)/2.0).floor();
   qr.push(Draw::rect(x,y,side,side,Color::from_hex(0xffffff)));
   for (j,row) in rows.iter().enumerate(){for (k,b) in row.bytes().enumerate(){if b==b'1'{qr.push(Draw::rect(x+k as f32*px,y+j as f32*px,px,px,Color::from_hex(0)));}}}
  }
  let mut info=vec![label("BTC / SUPPORT",0.0,8.0,right,24.0,16,AMBER)];
  if idx.is_some(){let font=if vp.width>=960{30}else if vp.width>=600{20}else{12};let line=font as f32+5.0;for (i,chunk) in address.as_bytes().chunks(14).enumerate(){info.push(label(std::str::from_utf8(chunk).unwrap(),0.0,45.0+i as f32*line,right,line,font,INK));}}
  else{info.push(label("Service 0.4.0?",0.0,50.0,right,40.0,14,MUTED));}
  let result=render_ui(vp.width,vp.height,col(props!(width:vp.width as f32,height:vp.height as f32,padding:6.0,gap:4.0,background:Color::from_hex(BG)),[
   row(props!(gap:4.0),[canvas(props!(width:left,height:area_h),qr),canvas(props!(width:right,height:area_h),info)]),key("donate_back","‹","",w,48.0,false,false)]));
  if result.clicks.contains_key("donate_back"){s.donate=false;request_frame();}return vec![];
 }

 if s.info{
  let t=&s.status["technical"];let lines=["RADIO / INFO".to_string(),t["input_audio"].as_str().unwrap_or("FFmpeg / madplay").to_string(),format!("{} s / {} B PCM",t["uptime_s"].as_u64().unwrap_or(0),t["pcm_bytes"].as_u64().unwrap_or(0)),format!("DATA {} ms / RETRY {}",t["last_data_ms"].as_u64().unwrap_or(0),t["reconnects"].as_u64().unwrap_or(0)),format!("WEB: {}",s.status["panel_url"].as_str().unwrap_or("No network address")),format!("RGB {} / #{} / v{}",if s.rgb{"ON"}else{"OFF"},s.status["remote_revision"].as_u64().unwrap_or(0),s.status["version"].as_str().unwrap_or("?"))];
  let h=(vp.height as f32-64.0)/6.0;let draws=lines.iter().enumerate().map(|(i,t)|label(t,0.0,i as f32*h,w,h,if i==0{20}else{12},INK)).collect::<Vec<_>>();
  let result=render_ui(vp.width,vp.height,col(props!(width:vp.width as f32,height:vp.height as f32,padding:6.0,gap:4.0,background:Color::from_hex(BG)),[canvas(props!(width:w,height:vp.height as f32-64.0),draws),key("info_back","‹","",w,48.0,false,false)]));if result.clicks.contains_key("info_back"){s.info=false;request_frame();}return vec![];
 }
 if s.settings&&!s.menu{
  let half=(w-4.0)/2.0;let h=(vp.height as f32-24.0)/4.0;
  let result=render_ui(vp.width,vp.height,col(props!(width:vp.width as f32,height:vp.height as f32,padding:6.0,gap:4.0,background:Color::from_hex(BG)),[
   row(props!(gap:4.0),[key("banks",tr(&s.lang,"presets"),"CZ EN DE ES JP USER",half,h,false,false),key("donate","BTC","SUPPORT / QR",half,h,false,false)]),
   row(props!(gap:4.0),[key("rgb_toggle",if s.rgb{"RGB ON"}else{"RGB OFF"},"REMOTE",half,h,s.rgb,false),key("rgb_level",&format!("RGB {}%",s.rgb_max),"+5% / 0-30%",half,h,false,false)]),
   row(props!(gap:4.0),[key("lang",&s.lang.to_uppercase(),"CZ EN DE ES JP",half,h,false,false),key("sleep",&format!("{} min",s.status["sleep_remaining"].as_u64().unwrap_or(0).div_ceil(60)),"SLEEP / OFF 15 30 60 90 120",half,h,false,false)]),
   row(props!(gap:4.0),[key("settings_back","‹","",half,h,false,false),key("info","INFO","WEB / PCM",half,h,false,false)])]));
  let hit=|k:&str|result.clicks.contains_key(k);
  if hit("settings_back"){s.settings=false;}if hit("banks"){s.menu=true;}if hit("donate"){s.donate=true;}if hit("info"){s.info=true;}
  if hit("sleep"){let n=s.status["sleep_remaining"].as_u64().unwrap_or(0).div_ceil(60);let next=[15u64,30,60,90,120].into_iter().find(|v|*v>n).unwrap_or(0);return vec![json!({"action":"sleep","minutes":next})];}
  if hit("rgb_toggle"){s.rgb=!s.rgb;if !s.rgb{led::stop();}}if hit("rgb_level"){s.rgb_max=if s.rgb_max>=30{0}else{(s.rgb_max+5).min(30)};}
  if hit("lang"){let langs=["en","cz","de","es","ja"];let i=langs.iter().position(|x|*x==s.lang).unwrap_or(0);s.lang=langs[(i+1)%5].into();}
  if hit("rgb_toggle")||hit("rgb_level")||hit("lang"){kv::set("panel-settings-v3",json!({"rgb":s.rgb,"max":s.rgb_max,"lang":s.lang}).to_string().as_bytes());}
  if !result.clicks.is_empty(){request_frame();}return vec![];
 }
 if s.list&&!s.settings&&!s.menu{
  let pages=s.stations.len().div_ceil(3).max(1);s.page=s.page.min(pages-1);let h=(vp.height as f32-72.0)/3.0;let mut rows=Vec::new();
  for j in 0..3{let i=s.page*3+j;rows.push(if let Some((name,url))=s.stations.get(i){key(&format!("station{i}"),name,"",w,h,s.status["wanted"]==true&&s.status["url"]==url.as_str(),false)}else{canvas(props!(width:w,height:h),[])})}
  let fw=(w-12.0)/4.0;rows.push(row(props!(gap:4.0),[key("list_back","‹","",fw,48.0,false,false),key("page_prev","-","",fw,48.0,false,s.page==0),key("page_next","+","",fw,48.0,false,s.page+1>=pages),key("list_settings","@settings","",fw,48.0,false,false)]));
  let result=render_ui(vp.width,vp.height,col(props!(width:vp.width as f32,height:vp.height as f32,padding:6.0,gap:4.0,background:Color::from_hex(BG)),rows));
  if result.clicks.contains_key("list_back"){s.list=false;}if result.clicks.contains_key("page_prev"){s.page=s.page.saturating_sub(1);}if result.clicks.contains_key("page_next"){s.page=(s.page+1).min(pages-1);}if result.clicks.contains_key("list_settings"){s.settings=true;}
  for i in 0..s.stations.len(){if result.clicks.contains_key(&format!("station{i}")){s.selected=i;s.list=false;s.muted=false;return vec![json!({"action":"mute","value":false}),play(s)];}}
  if !result.clicks.is_empty(){request_frame();}return vec![];
 }

 if s.menu {
 let cw=(w-8.0)/3.0;let ch=((vp.height as f32-72.0)/3.0).max(48.0);
 let bank=s.status["catalog"]["bank"].as_str().unwrap_or("");
 let can=s.status["api"].as_u64().unwrap_or(0)>=3;
 let result=render_ui(vp.width,vp.height,col(props!(width:vp.width as f32,height:vp.height as f32,padding:6.0,gap:4.0,background:Color::from_hex(BG)),[
 canvas(props!(width:w,height:48.0),[label(tr(&s.lang,"presets"),0.0,0.0,w,30.0,22,INK),label("GLOBAL",0.0,31.0,w,14.0,11,AMBER)]),
 row(props!(gap:4.0),[key("bank_cz","CZ","",cw,ch,bank=="cz",!can),key("bank_en","EN","",cw,ch,bank=="en",!can),key("bank_de","DE","",cw,ch,bank=="de",!can)]),
 row(props!(gap:4.0),[key("bank_es","ES","",cw,ch,bank=="es",!can),key("bank_ja","JP","",cw,ch,bank=="ja",!can),key("bank_custom","USER","",cw,ch,bank=="custom",!can)]),
 row(props!(gap:4.0),[key("back","‹","",(w-4.0)/2.0,ch,false,false),key("open_settings","@settings","",(w-4.0)/2.0,ch,false,false)])]));
 if result.clicks.contains_key("open_settings"){s.menu=false;s.settings=true;request_frame();}
 if result.clicks.contains_key("back"){s.menu=false;request_frame();}
 for bank in ["cz","en","de","es","ja","custom"]{if result.clicks.contains_key(&format!("bank_{bank}")){s.menu=false;s.settings=false;s.list=true;s.page=0;return vec![catalog_command(s,bank,false)];}}
 return vec![];
 }

 let state=s.status["status"].as_str().unwrap_or("waiting");let wanted=s.status["wanted"].as_bool().unwrap_or(false);
 let active_url=s.status["url"].as_str().unwrap_or("");
 let status_key=if !s.message.is_empty(){s.message.as_str()}else{state};
 let status_color=if status_key=="error"{0xe89581}else if state=="playing"{AMBER}else{MUTED};
 let title=if wanted{s.status["name"].as_str().unwrap_or(&s.stations[s.selected].0)}else{&s.stations[s.selected].0};
 let mut head_draws=vec![
 label("ROLID  /  RADIO",0.0,0.0,w*0.7,12.0,10,MUTED),
 label(if s.status["aac"]==true{"AAC/MP3"}else{"MP3"},w-52.0,0.0,52.0,12.0,10,MUTED),
 label(title,0.0,14.0,w,l.head-40.0,if w<400.0{24}else{36},INK),
 label(tr(&s.lang,status_key),0.0,l.head-16.0,w,14.0,12,status_color),
 ];
 for channel in 0..2{let bar_w=(w-8.0)/2.0;let x=channel as f32*(bar_w+8.0);let value=if measured{s.status["meter"]["rms"][channel].as_u64().unwrap_or(0) as f32/1000.0}else{0.0};
  let db=if value>0.0{(20.0*value.log10()+48.0)/48.0}else{0.0};
  for i in 0..18{head_draws.push(Draw::rect(x+i as f32*bar_w/18.0,l.head-23.0,bar_w/18.0-2.0,5.0,Color::from_hex(if measured&&i as f32/18.0<db{if i>15{0xe89581}else{AMBER}}else{LINE})));}
 }
 let head=touchable("catalog",props!(width:w,height:l.head),head_draws);
 let count=if vp.width>=960{12}else if vp.width>=600{8}else{5};let pw=(w-gap*(count-1) as f32)/count as f32;let mut presets=Vec::new();
 for i in 0..count {let station=s.stations.get(i);let enabled=station.is_some();let selected=station.is_some_and(|v|wanted&&v.1==active_url);
 let short=station.map(|v|v.0.chars().take(if w<400.0{7}else{14}).collect::<String>()).unwrap_or_else(||"—".into());
 presets.push(key(&format!("p{i}"),&format!("{:02}",i+1),&short,pw,l.preset,selected,!enabled));}
 let side=(w*0.19).max(48.0);let middle=w-side*2.0-gap*2.0;
 let transport=if vp.width<600{row(props!(gap:gap),[key("open_list","@list","",side,l.transport,false,false),key("toggle",tr(&s.lang,if wanted{"stop"}else{"play"}),"",middle,l.transport,wanted,false),key("open_settings","@settings","",side,l.transport,false,false)])}else{
 let nav=(w*0.10).max(48.0);let mid=w-4.0*nav-4.0*gap;row(props!(gap:gap),[key("open_list","@list","",nav,l.transport,false,false),key("prev","‹","",nav,l.transport,false,false),key("toggle",tr(&s.lang,if wanted{"stop"}else{"play"}),"",mid,l.transport,wanted,false),key("next","›","",nav,l.transport,false,false),key("open_settings","@settings","",nav,l.transport,false,false)])};
 let volume=row(props!(gap:gap),[
 key("down","-","",side,l.volume,false,false),
 key("mute",&format!("{:02}%",s.volume),tr(&s.lang,if s.muted{"unmute"}else{"mute"}),middle,l.volume,s.muted,false),
 key("up","+","",side,l.volume,false,false)]);
 let result=render_ui(vp.width,vp.height,col(props!(width:vp.width as f32,height:vp.height as f32,padding:6.0,gap:gap,background:Color::from_hex(BG)),[head,row(props!(gap:gap),presets),transport,volume]));
 let hit=|id:&str|result.clicks.contains_key(id);let mut out=Vec::new();if hit("catalog")||hit("open_list"){s.list=true;s.page=s.selected/3;request_frame();}if hit("open_settings"){s.settings=true;request_frame();}
 for i in 0..count{if hit(&format!("p{i}"))&&i<s.stations.len(){s.selected=i;s.muted=false;out.push(json!({"action":"mute","value":false}));out.push(play(s));}}
 if hit("prev")||hit("next"){s.selected=if hit("prev"){(s.selected+s.stations.len()-1)%s.stations.len()}else{(s.selected+1)%s.stations.len()};if wanted{out.push(play(s));}request_frame();}
 if hit("toggle"){if wanted{out.push(json!({"action":"stop"}));}else{s.muted=false;out.push(json!({"action":"mute","value":false}));out.push(play(s));}}
 if hit("mute"){s.muted=!s.muted;out.push(json!({"action":"mute","value":s.muted}));}
 if hit("down")||hit("up"){s.volume=if hit("down"){s.volume.saturating_sub(5)}else{(s.volume+5).min(100)};out.push(json!({"action":"volume","value":s.volume}));}
 out
 });for command in commands {send(Some(command));}request_frame_after(200);
}
