mod music;mod access;mod alarm;
mod donation;mod meter;mod web;mod telemetry;
use std::{io::{Read,Write}, net::{TcpListener,TcpStream},process::{Child,Command,Stdio},sync::{Arc,atomic::{AtomicU64,AtomicBool,Ordering}},time::{Duration,Instant}};
use serde::{Deserialize,Serialize};
const ADDR:&str="127.0.0.1:18765";
const AUTH:&str="Rolid-radio-v1";
#[derive(Default,Serialize,Deserialize)]
struct Prefs {url:String,name:String,volume:u8}
#[derive(Clone,Default,Serialize,Deserialize)] struct Catalog {revision:u64,bank:String,stations:Vec<Station>}
#[derive(Clone,Serialize,Deserialize)] struct Station {name:String,url:String,#[serde(default)]logo:String}
fn data_dir()->std::path::PathBuf {std::env::var_os("ROLID_RADIO_DATA_DIR").map(std::path::PathBuf::from).unwrap_or_else(||"/mnt/data/rolid-radio".into())}
fn supported(u:&str)->bool {valid_url(u)&&url::Url::parse(u).is_ok_and(|v|![".m3u",".m3u8",".pls"].iter().any(|s|v.path().to_lowercase().ends_with(s)))}
struct Player {info:Arc<std::sync::Mutex<telemetry::Info>>,pcm:bool,levels:Arc<[AtomicU64;4]>,fetch:Child,decode:Child,bytes:Arc<AtomicU64>,started:Instant,last_bytes:u64,last_data:Instant}
impl Drop for Player {fn drop(&mut self){let _=self.fetch.kill();let _=self.decode.kill();let _=self.fetch.wait();let _=self.decode.wait();}}
fn sink()->String{std::env::var("ROLID_RADIO_APLAY").unwrap_or_else(|_|"/usr/bin/aplay".into())}
mod network;
struct Radio {alarm:Arc<AtomicBool>,music:music::Music,radio_source:Option<(String,String)>,panel_url:Option<String>,panel_interface:Option<String>,network_refresh:Instant,remote_revision:u64,sleep_until:Option<Instant>,banks:std::collections::BTreeMap<String,Vec<Station>>,donation:Option<String>,ffmpeg:bool,catalog:Catalog,prefs:Prefs,wanted:bool,muted:bool,player:Option<Player>,retry:Instant,failures:u32,error:String,duck_until:Instant}
fn valid_url(s:&str)->bool {url::Url::parse(s).is_ok_and(|u|matches!(u.scheme(),"http"|"https") && u.host_str().is_some() && u.username().is_empty() && u.password().is_none() && s.len()<=2048 && !s.chars().any(char::is_control))}
impl Radio {
 fn new()->Self{let prefs=std::fs::read(data_dir().join("state.json")).ok().and_then(|b|serde_json::from_slice(&b).ok()).unwrap_or(Prefs{volume:60,..Prefs::default()});let saved=std::fs::read(data_dir().join("catalog-store.json")).ok().and_then(|b|serde_json::from_slice::<serde_json::Value>(&b).ok());let catalog:Catalog=saved.as_ref().and_then(|v|serde_json::from_value(v["catalog"].clone()).ok()).or_else(||std::fs::read(data_dir().join("catalog.json")).ok().and_then(|b|serde_json::from_slice(&b).ok())).unwrap_or_default();let mut banks:std::collections::BTreeMap<String,Vec<Station>>=saved.as_ref().and_then(|v|serde_json::from_value(v["banks"].clone()).ok()).unwrap_or_default();if !catalog.stations.is_empty(){banks.entry(catalog.bank.clone()).or_insert_with(||catalog.stations.clone());}let (panel_url,panel_interface)=network::panel();Self{alarm:alarm::start(),music:music::Music::new(),radio_source:None,panel_url,panel_interface,network_refresh:Instant::now(),remote_revision:0,sleep_until:None,banks,donation:donation::load_or_create(&data_dir()).map_err(|e|eprintln!("Donation: {e}")).ok(),ffmpeg:std::path::Path::new("/usr/bin/ffmpeg").is_file()&&std::path::Path::new(&sink()).is_file(),catalog,prefs,wanted:false,muted:false,player:None,retry:Instant::now(),failures:0,error:String::new(),duck_until:Instant::now()}}
 fn persist(&self){let (url,name)=self.radio_source.as_ref().map(|(u,n)|(u.as_str(),n.as_str())).unwrap_or((&self.prefs.url,&self.prefs.name));if let Ok(b)=serde_json::to_vec(&serde_json::json!({"url":url,"name":name,"volume":self.prefs.volume})){let p=data_dir().join("state.json.tmp");if std::fs::write(&p,b).is_ok(){let _=std::fs::rename(&p,data_dir().join("state.json"));}}}
 fn remember_position(&mut self){if self.music.current.is_some(){self.music.offset+=self.player.as_ref().map(|p|p.bytes.load(Ordering::Relaxed) as f64/192000.).unwrap_or(0.);}}
 fn reset(&mut self){self.player=None;self.retry=Instant::now();self.failures=0;self.error.clear();}
 fn start(&self)->Result<Player,String>{
  if self.ffmpeg{return self.start_pcm()}
  let mut fetch=Command::new("/usr/bin/wget").args(["-q","-O","-",&self.prefs.url]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|e|format!("wget: {e}"))?;
  let db=(i32::from(self.prefs.volume)*40+50)/100-40;
  let decode=Command::new("/usr/bin/madplay").args(["-q","-A",&db.to_string(),"-"]).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
  let mut decode=match decode {Ok(c)=>c,Err(e)=>{let _=fetch.kill();let _=fetch.wait();return Err(format!("madplay: {e}"));}};
  let mut input=fetch.stdout.take().unwrap();let mut output=decode.stdin.take().unwrap();let bytes=Arc::new(AtomicU64::new(0));let count=bytes.clone();
  std::thread::spawn(move||{let mut buf=[0;8192];loop{let n=match input.read(&mut buf){Ok(0)|Err(_)=>break,Ok(n)=>n};if output.write_all(&buf[..n]).is_err(){break}count.fetch_add(n as u64,Ordering::Relaxed);}});
  Ok(Player{info:Arc::new(std::sync::Mutex::new(telemetry::Info::default())),pcm:false,levels:Arc::new(std::array::from_fn(|_|AtomicU64::new(0))),fetch,decode,bytes,started:Instant::now(),last_bytes:0,last_data:Instant::now()})
 }
 fn start_pcm(&self)->Result<Player,String>{
  let db=(i32::from(self.prefs.volume)*40+50)/100-40;
  let mut cmd=Command::new("/usr/bin/ffmpeg");if self.music.current.is_some(){cmd.args(["-ss",&format!("{:.3}",self.music.offset)]);}let mut fetch=cmd.args(["-nostdin","-hide_banner","-loglevel","info","-nostats","-stats_period","0.2","-progress","pipe:2","-rw_timeout","15000000","-protocol_whitelist",if self.music.current.is_some()&&!valid_url(&self.prefs.url){"file"}else{"http,https,tcp,tls,crypto"},"-i",&self.prefs.url,"-vn","-af",&format!("volume={db}dB"),"-ac","2","-ar","48000","-c:a","pcm_s16le","-f","s16le","pipe:1"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|format!("ffmpeg: {e}"))?;
  let decode=Command::new(sink()).args(["-q","-D","plughw:0,0","-t","raw","-f","S16_LE","-r","48000","-c","2","--buffer-time=60000","--period-time=20000"]).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
  let mut decode=match decode{Ok(c)=>c,Err(e)=>{let _=fetch.kill();let _=fetch.wait();return Err(format!("aplay: {e}"));}};
  let info=Arc::new(std::sync::Mutex::new(telemetry::Info::default()));telemetry::watch(fetch.stderr.take().unwrap(),info.clone());
  let mut input=fetch.stdout.take().unwrap();let mut output=decode.stdin.take().unwrap();
  use std::os::fd::AsRawFd;unsafe{libc::fcntl(input.as_raw_fd(),libc::F_SETPIPE_SZ,4096);libc::fcntl(output.as_raw_fd(),libc::F_SETPIPE_SZ,4096);}
  let bytes=Arc::new(AtomicU64::new(0));let count=bytes.clone();let levels=Arc::new(std::array::from_fn(|_|AtomicU64::new(0)));let shared:Arc<[AtomicU64;4]>=levels.clone();
  std::thread::spawn(move||{let mut buf=[0;3840];let mut sum=[0u64;2];let mut peaks=[0u64;2];let mut samples=0;loop{ // 20 ms stereo PCM, sink backpressure bounds decoding.
   if input.read_exact(&mut buf).is_err(){break}
   if output.write_all(&buf).is_err(){break}
   let (rms,peak)=meter::levels(&buf);
   for i in 0..2{sum[i]+=rms[i]*rms[i];peaks[i]=peaks[i].max(peak[i]);}samples+=1;if samples==5{for i in 0..2{shared[i].store(((sum[i]/5) as f64).sqrt() as u64,Ordering::Relaxed);shared[i+2].store(peaks[i],Ordering::Relaxed);}sum=[0;2];peaks=[0;2];samples=0;}
   count.fetch_add(buf.len() as u64,Ordering::Relaxed);
  }});
  Ok(Player{info,pcm:true,levels,fetch,decode,bytes,started:Instant::now(),last_bytes:0,last_data:Instant::now()})
 }
 fn tick(&mut self){self.music.expire();if self.alarm.load(Ordering::Relaxed){if self.player.is_some(){self.remember_position();self.player=None;}self.duck_until=self.duck_until.max(Instant::now()+Duration::from_secs(6));}if self.network_refresh.elapsed()>=Duration::from_secs(5){let (url,interface)=network::panel();self.panel_url=url;self.panel_interface=interface;self.network_refresh=Instant::now();}if self.sleep_until.is_some_and(|t|Instant::now()>=t){self.sleep_until=None;self.wanted=false;self.reset();}
  if self.music.current.is_some()&&self.music.paused{return}if !self.wanted||self.muted||self.prefs.volume==0||Instant::now()<self.duck_until {self.player=None;return}
  let mut failure=None;
  if let Some(p)=self.player.as_mut(){
   if self.music.current.is_none(){if let Ok(Some(s))=p.fetch.try_wait(){failure=Some(format!("Stream ended: {s}"));}}
   if let Ok(Some(s))=p.decode.try_wait(){failure=Some(format!("Decoder ended: {s}"));}
   let n=p.bytes.load(Ordering::Relaxed);if n!=p.last_bytes{p.last_bytes=n;p.last_data=Instant::now();}
   if p.last_data.elapsed()>Duration::from_secs(20){failure=Some("No stream data for 20 seconds".into());}
  }
  if self.music.current.is_some(){if let Some(e)=failure.take(){let clean=self.player.as_mut().is_some_and(|p|p.decode.try_wait().ok().flatten().is_some_and(|s|s.success())&&p.fetch.try_wait().ok().flatten().is_some_and(|s|s.success()));self.player=None;if clean{if let Some(name)=self.music.next(false,true){self.prefs.name=name;self.prefs.url=self.music.input().unwrap();self.retry=Instant::now();}else{self.wanted=false;self.music.offset=0.;return}}else{self.wanted=false;self.error=e;return}}}
  if let Some(e)=failure {self.player=None;self.failures+=1;self.error=e;self.retry=Instant::now()+Duration::from_secs((3*u64::from(self.failures)).min(30));}
  if self.player.is_none() && Instant::now()>=self.retry {match self.start(){Ok(p)=>{self.player=Some(p);},Err(e)=>{self.error=e;self.failures+=1;self.retry=Instant::now()+Duration::from_secs(5);}}}
 }
 fn levels(&self)->serde_json::Value{serde_json::json!({"meter":self.player.as_ref().filter(|p|p.pcm&&p.last_data.elapsed()<Duration::from_millis(500)).map(|p|serde_json::json!({"rms":[p.levels[0].load(Ordering::Relaxed),p.levels[1].load(Ordering::Relaxed)],"peak":[p.levels[2].load(Ordering::Relaxed),p.levels[3].load(Ordering::Relaxed)]}))})}
 fn status(&self)->serde_json::Value{
  let status=if self.music.current.is_some()&&self.music.paused{"paused"}else if !self.wanted{"stopped"}else if Instant::now()<self.duck_until{"alarm"}else if self.muted||self.prefs.volume==0{"muted"}else if let Some(p)=&self.player{if p.bytes.load(Ordering::Relaxed)>0 && p.started.elapsed()>Duration::from_millis(150){"playing"}else{"connecting"}}else{"retrying"};
  serde_json::json!({"api":7,"version":"0.7.2","access_enabled":access::enabled(),"alarm_monitor":"local-madplay","mode":if self.music.current.is_some(){"music"}else{"radio"},"music":{"last_id":self.music.last,"tracks":self.music.brief(),"groups":self.music.groups(),"queue":self.music.queue,"id":self.music.current,"position":self.music.offset+self.player.as_ref().filter(|_|self.music.current.is_some()).map(|p|p.bytes.load(Ordering::Relaxed) as f64/192000.).unwrap_or(0.),"repeat":self.music.repeat,"shuffle":self.music.shuffle},"upload":self.music.upload_status(),"remote_revision":self.remote_revision,"panel_url":self.panel_url,"panel_interface":self.panel_interface,"sleep_remaining":self.sleep_until.map(|t|t.saturating_duration_since(Instant::now()).as_secs()),"technical":self.player.as_ref().map(|p|{let i=p.info.lock().unwrap();serde_json::json!({"decoder":if p.pcm{"ffmpeg"}else{"madplay"},"input_audio":i.input_audio,"decoded_us":i.decoded_us,"speed":i.speed,"last_error":i.last_error,"uptime_s":p.started.elapsed().as_secs(),"pcm_bytes":if p.pcm{Some(p.bytes.load(Ordering::Relaxed))}else{None},"network_bytes":if p.pcm{None}else{Some(p.bytes.load(Ordering::Relaxed))},"last_data_ms":p.last_data.elapsed().as_millis() as u64,"reconnects":self.failures,"output_rate":if p.pcm{Some(48000)}else{None},"cache_percent":null})}),"aac":self.ffmpeg,"donation_address":self.donation,"meter":self.player.as_ref().filter(|p|p.pcm&&p.last_data.elapsed()<Duration::from_millis(500)).map(|p|serde_json::json!({"rms":[p.levels[0].load(Ordering::Relaxed),p.levels[1].load(Ordering::Relaxed)],"peak":[p.levels[2].load(Ordering::Relaxed),p.levels[3].load(Ordering::Relaxed)]})),"bank_names":self.banks.keys().collect::<Vec<_>>(),"catalog":self.catalog,"status":status,"name":self.prefs.name,"url":self.prefs.url,"volume":self.prefs.volume,"muted":self.muted,"wanted":self.wanted,"error":self.error,"bytes":self.player.as_ref().map(|p|p.bytes.load(Ordering::Relaxed)).unwrap_or(0)})
 }
 fn command(&mut self,v:serde_json::Value)->Result<(),String>{
  match v["action"].as_str().unwrap_or("") {
   "levels"=>{},"access_get"=>{},"access_set"=>{access::update(&v)?;},
   "music_network_add"|"music_folder_create"|"music_folder_rename"|"music_folder_delete"|"music_move"|"music_playlist_save"|"music_playlist_delete"=>{self.music.organize(&v)?;},
   "music_library"|"music_upload_begin"|"music_upload_chunk"|"music_upload_finish"|"music_upload_cancel"|"music_edit"|"music_delete"=>{self.music.edit(&v)?;},
   "music_play"=>{if !self.ffmpeg{return Err("Local music requires FFmpeg and aplay".into())}self.music.set_queue(&v)?;let id=v["id"].as_str().map(str::to_owned).unwrap_or_else(||self.music.queue[0].clone());let name=self.music.select(&id)?;if self.radio_source.is_none(){self.radio_source=Some((self.prefs.url.clone(),self.prefs.name.clone()));}self.reset();self.prefs.url=self.music.input().unwrap();self.prefs.name=name;self.wanted=true;self.muted=false;},
   "music_pause"=>{if self.music.current.is_none(){return Err("Not in music mode".into())}self.remember_position();self.reset();self.music.paused=true;},
   "music_resume"=>{if self.music.current.is_none(){let id=self.music.last.clone().ok_or("No previous track; open Library")?;self.music.set_queue(&serde_json::json!({"id":id}))?;let name=self.music.select(&id)?;self.radio_source=Some((self.prefs.url.clone(),self.prefs.name.clone()));self.reset();self.prefs.url=self.music.input().unwrap();self.prefs.name=name;}if self.player.is_none(){self.music.paused=false;self.wanted=true;self.reset();}},
   "music_next"|"music_previous"=>{if self.music.current.is_none(){return Err("Not in music mode".into())}if let Some(name)=self.music.next(v["action"]=="music_previous",false){self.reset();self.prefs.name=name;self.prefs.url=self.music.input().unwrap();self.wanted=true;}},
   "music_options"=>{self.music.options(v["repeat"].as_bool().ok_or("Repeat must be boolean")?,v["shuffle"].as_bool().ok_or("Shuffle must be boolean")?)?;},
   "radio_mode"=>{self.player=None;self.music.current=None;self.music.paused=false;self.music.offset=0.;self.wanted=false;if let Some((url,name))=self.radio_source.take(){self.prefs.url=url;self.prefs.name=name;}},
   "catalog"=>{
    if v["seed"].as_bool()==Some(true)&&self.catalog.revision>0{return Ok(())}
    if v["revision"].as_u64()!=Some(self.catalog.revision){return Err("Catalog changed; refresh and retry".into())}
    let bank=v["bank"].as_str().unwrap_or("custom");if bank.trim().is_empty()||bank.chars().count()>32||bank.chars().any(char::is_control){return Err("Unknown bank".into())}
    let stations:Vec<Station>=if v["edit"]==true || (bank=="custom" && v["stations"].as_array().is_some_and(|v|!v.is_empty())) {serde_json::from_value(v["stations"].clone()).map_err(|_|"Invalid stations")?}else if let Some(stations)=self.banks.get(bank){stations.clone()}else if bank=="custom"&&!self.catalog.stations.is_empty(){self.catalog.stations.clone()}else{
      let banks:serde_json::Value=serde_json::from_str(include_str!("../../stations.json")).unwrap();
      serde_json::from_value(banks[bank].clone()).map_err(|_|"Unknown station bank")?
    };
    if stations.iter().map(|s|s.logo.len()).sum::<usize>()>65536{return Err("Group logos exceed 32 KiB".into())}if stations.is_empty()||stations.len()>32||stations.iter().any(|s|s.name.trim().is_empty()||s.name.chars().count()>60||s.url.len()>1024||!supported(&s.url)||!valid_logo(&s.logo)){return Err("Use 1..32 named direct MP3/AAC streams (playlists unsupported)".into())}
    let catalog=Catalog{revision:self.catalog.revision+1,bank:bank.into(),stations};
    let mut banks=self.banks.clone();if let Some(old)=v["rename_from"].as_str(){if old!=bank&&banks.contains_key(bank){return Err("Group exists".into())}banks.remove(old);}banks.insert(bank.into(),catalog.stations.clone());let bytes=serde_json::to_vec(&serde_json::json!({"catalog":catalog,"banks":banks})).map_err(|e|e.to_string())?;
    let tmp=data_dir().join("catalog-store.json.tmp");std::fs::write(&tmp,bytes).and_then(|_|std::fs::rename(&tmp,data_dir().join("catalog-store.json"))).map_err(|e|format!("Save catalog: {e}"))?;
    self.catalog=catalog;self.banks=banks;if let Some(station)=self.catalog.stations.iter().find(|s|s.url==self.prefs.url){self.prefs.name=station.name.clone();self.persist();}
   },
   "rgb_test"=>{},
   "sleep"=>{let n=v["minutes"].as_u64().filter(|n|*n<=240).ok_or("Sleep: 0..240 minutes")?;self.sleep_until=if n==0{None}else{Some(Instant::now()+Duration::from_secs(n*60))};},
   "play"=>{let u=v["url"].as_str().ok_or("Missing URL")?;if !supported(u){return Err("Use direct MP3/AAC; playlists are not supported".into())}if !self.ffmpeg&&url::Url::parse(u).is_ok_and(|v|v.path().to_lowercase().ends_with(".aac")){return Err("AAC requires FFmpeg; run enable-aac.sh".into())}let n=v["name"].as_str().unwrap_or("Radio");if n.len()>200{return Err("Name too long".into())}self.music.current=None;self.music.paused=false;self.radio_source=None;self.prefs.url=u.into();self.prefs.name=n.into();self.wanted=true;self.reset();self.persist();},
   "stop"=>{self.music.paused=false;self.music.offset=0.;self.sleep_until=None;self.wanted=false;self.reset();},
   "volume"=>{let n=v["value"].as_u64().filter(|n|*n<=100).ok_or("Volume must be 0..100")? as u8;self.remember_position();self.prefs.volume=n;self.reset();self.persist();},
   "mute"=>{let muted=v["value"].as_bool().ok_or("Mute must be boolean")?;self.remember_position();self.muted=muted;self.reset();},
   "duck"=>{self.remember_position();self.duck_until=Instant::now()+Duration::from_secs(8);self.player=None;},
   _=>return Err("Unknown action".into())
  }Ok(())
 }
}
fn request(s:&mut TcpStream)->Result<(String,String,String,Vec<u8>),String>{
 s.set_read_timeout(Some(Duration::from_millis(400))).map_err(|e|e.to_string())?;s.set_write_timeout(Some(Duration::from_secs(1))).map_err(|e|e.to_string())?;
 let mut buf=Vec::new();let split=loop{if buf.len()>8192{return Err("Header too large".into())}let mut byte=[0];s.read_exact(&mut byte).map_err(|e|e.to_string())?;buf.push(byte[0]);if buf.ends_with(b"\r\n\r\n"){break buf.len()}};
 let head=std::str::from_utf8(&buf).map_err(|_|"Bad header")?;let mut lines=head.split("\r\n");let mut first=lines.next().unwrap_or("").split_whitespace();let method=first.next().unwrap_or("").to_string();let path=first.next().unwrap_or("").to_string();let mut auth=String::new();let mut len=0;
 for l in lines{if let Some((k,v))=l.split_once(':'){if k.eq_ignore_ascii_case("authorization"){auth=v.trim().into()}if k.eq_ignore_ascii_case("content-length"){len=v.trim().parse::<usize>().map_err(|_|"Bad length")?}if k.eq_ignore_ascii_case("transfer-encoding"){return Err("Transfer encoding unsupported".into())}}}
 if len>262144{return Err("Body too large".into())}buf.resize(split+len,0);s.read_exact(&mut buf[split..]).map_err(|e|e.to_string())?;Ok((method,path,auth,buf[split..].to_vec()))
}
fn reply(s:&mut TcpStream,code:&str,body:serde_json::Value){let data=body.to_string();let _=write!(s,"HTTP/1.1 {code}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n{data}",data.len());}
fn duck()->Result<(),String>{let mut s=TcpStream::connect_timeout(&ADDR.parse().unwrap(),Duration::from_secs(1)).map_err(|e|e.to_string())?;s.set_read_timeout(Some(Duration::from_secs(2))).map_err(|e|e.to_string())?;let body=r#"{"action":"duck"}"#;write!(s,"POST /command HTTP/1.1\r\nHost: localhost\r\nAuthorization: {AUTH}\r\nContent-Length: {}\r\n\r\n{body}",body.len()).map_err(|e|e.to_string())?;let mut res=String::new();s.read_to_string(&mut res).map_err(|e|e.to_string())?;if res.starts_with("HTTP/1.1 200"){Ok(())}else{Err(res)}}
fn main(){if std::env::args().any(|a|a=="--print-access-key"||a=="--print-password"){match access::info(){Ok(v)=>println!("{}",v["access"]["key"].as_str().unwrap_or("")),Err(e)=>{eprintln!("{e}");std::process::exit(1)}}return}if std::env::args().any(|a|a=="--duck"){if duck().is_err(){std::process::exit(1)}return}let listener=TcpListener::bind(ADDR).expect("Bind localhost port 18765");listener.set_nonblocking(true).unwrap();let quit=Arc::new(AtomicBool::new(false));
 for sig in [signal_hook::consts::SIGTERM,signal_hook::consts::SIGINT] {signal_hook::flag::register(sig,quit.clone()).unwrap();}
 let mut radio=Radio::new();let web_requests=if std::env::args().any(|a|a=="--web"){Some(web::start().expect("Start web panel"))}else{None};while !quit.load(Ordering::Relaxed){
 radio.tick();if let Some(rx)=&web_requests{for msg in rx.try_iter().take(8){let result=match msg.command{Some(v)=>{let security=v["action"]=="access_get"||v["action"]=="access_set";let library=v["action"]=="music_library";let levels=v["action"]=="levels";let test=v["action"]=="rgb_test";let before=(radio.prefs.url.clone(),radio.prefs.name.clone(),radio.prefs.volume,radio.muted,radio.wanted,radio.catalog.revision,radio.sleep_until);radio.command(v).map(|_|{let after=(radio.prefs.url.clone(),radio.prefs.name.clone(),radio.prefs.volume,radio.muted,radio.wanted,radio.catalog.revision,radio.sleep_until);if test||before!=after{radio.remote_revision=radio.remote_revision.saturating_add(1);}if security{access::info().unwrap_or(serde_json::json!({"error":"Access settings unavailable"}))}else if levels{radio.levels()}else if library{radio.music.list()}else{radio.status()}})},None=>Ok(radio.status())};let _=msg.reply.send(result);}}match listener.accept(){Ok((mut s,peer))=>{if !peer.ip().is_loopback(){continue}match request(&mut s){Ok((method,path,auth,body))=>{if method=="GET"&&path=="/levels"{reply(&mut s,"200 OK",radio.levels())}else if method=="GET"&&path=="/status"{reply(&mut s,"200 OK",radio.status())}else if method=="POST"&&path=="/command"&&auth==AUTH {match serde_json::from_slice(&body).map_err(|e|e.to_string()).and_then(|v:serde_json::Value|{let security=v["action"]=="access_get"||v["action"]=="access_set";let library=v["action"]=="music_library";radio.command(v).and_then(|_|if security{access::info()}else if library{Ok(radio.music.list())}else{Ok(radio.status())})}){Ok(result)=>reply(&mut s,"200 OK",result),Err(e)=>reply(&mut s,"400 Bad Request",serde_json::json!({"error":e}))}}else{reply(&mut s,"403 Forbidden",serde_json::json!({"error":"Use authorized POST /command or GET /status"}))}},Err(e)=>reply(&mut s,"400 Bad Request",serde_json::json!({"error":e}))}},Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>std::thread::sleep(Duration::from_millis(10)),Err(e)=>{eprintln!("{e}");std::thread::sleep(Duration::from_secs(1));}}
}}
#[cfg(test)] mod tests{use super::*;#[test]fn url_validation(){assert!(valid_url("https://rozhlas.stream/radiozurnal_mp3_128.mp3"));for s in ["file:///etc/passwd","-O /tmp/x","https://u:p@example.com","http://x\n"]{assert!(!valid_url(s));}}#[test]fn controls(){let mut r=Radio::new();r.panel_url=Some("stale-address".into());r.network_refresh=Instant::now()-Duration::from_secs(6);r.tick();assert_ne!(r.panel_url.as_deref(),Some("stale-address"));r.sleep_until=Some(Instant::now()-Duration::from_secs(1));r.wanted=true;r.tick();assert!(!r.wanted&&r.sleep_until.is_none());r.command(serde_json::json!({"action":"mute","value":true})).unwrap();assert!(r.muted);assert!(r.command(serde_json::json!({"action":"volume","value":101})).is_err());r.command(serde_json::json!({"action":"duck"})).unwrap();assert!(r.duck_until>Instant::now());}}

#[cfg(test)] mod lifecycle_tests {
 use super::*;
 #[test] fn dropping_player_terminates_both_children(){
 let mut fetch=Command::new("sleep").arg("60").stdout(Stdio::piped()).spawn().unwrap();
 let mut decode=Command::new("sleep").arg("60").stdout(Stdio::piped()).spawn().unwrap();
 let (tx,rx)=std::sync::mpsc::channel();
 for mut pipe in [fetch.stdout.take().unwrap(),decode.stdout.take().unwrap()]{let tx=tx.clone();std::thread::spawn(move||{let mut data=Vec::new();let r=pipe.read_to_end(&mut data);let _=tx.send(r);});}
 let p=Player{info:Arc::new(std::sync::Mutex::new(telemetry::Info::default())),pcm:false,levels:Arc::new(std::array::from_fn(|_|AtomicU64::new(0))),fetch,decode,bytes:Arc::new(AtomicU64::new(0)),started:Instant::now(),last_bytes:0,last_data:Instant::now()};
 drop(p);
 for _ in 0..2{assert_eq!(rx.recv_timeout(Duration::from_secs(2)).expect("child stdout must close promptly").unwrap(),0);}
 }
}

fn valid_logo(s:&str)->bool{if s.is_empty(){return true}if s.len()>16384||s.len()<48||s.len()%2!=0||!s.bytes().all(|b|b.is_ascii_hexdigit()){return false}let b:Vec<u8>=(0..48).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();b.starts_with(b"\x89PNG\r\n\x1a\n")&&(1..=96).contains(&u32::from_be_bytes(b[16..20].try_into().unwrap()))&&(1..=96).contains(&u32::from_be_bytes(b[20..24].try_into().unwrap()))}
