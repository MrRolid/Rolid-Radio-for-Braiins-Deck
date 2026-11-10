//! Persistent, bounded MP3 library. User filenames are never filesystem paths.
use std::{fs,io::{Read,Write},path::PathBuf,time::Instant};
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
pub const LIMIT:u64=1024*1024*1024;
const RESERVE:u64=512*1024*1024;
#[derive(Clone,Serialize,Deserialize)] pub struct Track{pub id:String,pub title:String,pub artist:String,pub album:String,pub bytes:u64,#[serde(default)]pub source:String,#[serde(default)]pub folder:String}
#[derive(Clone,Serialize,Deserialize)] struct Folder{id:String,name:String,parent:String}
#[derive(Clone,Serialize,Deserialize)] struct Playlist{id:String,name:String,tracks:Vec<String>}
#[derive(Clone,Default,Serialize,Deserialize)] struct Index{#[serde(default)]folders:Vec<Folder>,#[serde(default)]playlists:Vec<Playlist>,tracks:Vec<Track>,#[serde(default)]repeat:bool,#[serde(default)]shuffle:bool}
struct Upload{folder:String,id:String,title:String,size:u64,written:u64,touched:Instant}
pub struct Music{pub queue:Vec<String>,pub last:Option<String>,root:PathBuf,index:Index,upload:Option<Upload>,pub current:Option<String>,pub offset:f64,pub paused:bool,pub repeat:bool,pub shuffle:bool,error:Option<String>}
fn id_ok(s:&str)->bool{s.len()==32&&s.bytes().all(|c|c.is_ascii_hexdigit())}
impl Music{
 pub fn new()->Self{
  let root=super::data_dir().join("music");let mut error=fs::create_dir_all(&root).err().map(|e|e.to_string());
  let index:Index=match fs::read(root.join("library.json")){Ok(b)=>match serde_json::from_slice(&b){Ok(i)=>i,Err(e)=>{error=Some(format!("Library index error: {e}"));Index::default()}},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Index::default(),Err(e)=>{error=Some(e.to_string());Index::default()}};
  if index.tracks.iter().any(|t|!id_ok(&t.id)){error=Some("Invalid track ID in library index".into());}
  if let Ok(files)=fs::read_dir(&root){for f in files.flatten(){if f.path().extension().is_some_and(|s|s=="part"){let _=fs::remove_file(f.path());}}}
  let last=fs::read_to_string(root.join("last-track")).ok().filter(|id|index.tracks.iter().any(|t|&t.id==id));Self{last,queue:vec![],repeat:index.repeat,shuffle:index.shuffle,root,index,upload:None,current:None,offset:0.,paused:false,error}
 }
 fn path(&self,id:&str)->PathBuf{self.root.join(format!("{id}.mp3"))}
 fn check(&self)->Result<(),String>{if let Some(e)=&self.error{Err(e.clone())}else{Ok(())}}
 pub fn input(&self)->Option<String>{self.current.as_ref().and_then(|id|self.index.tracks.iter().find(|t|&t.id==id).map(|t|if t.source.is_empty(){self.path(id).to_string_lossy().into_owned()}else{t.source.clone()}))}
 pub fn select(&mut self,id:&str)->Result<String,String>{self.check()?;let t=self.index.tracks.iter().find(|t|t.id==id).ok_or("Track not found")?;if t.source.is_empty()&&!self.path(id).is_file(){return Err("Track file missing".into())}let title=t.title.clone();fs::write(self.root.join("last-track"),id).map_err(|e|e.to_string())?;self.last=Some(id.into());self.current=Some(id.into());self.offset=0.;self.paused=false;Ok(title)}
 pub fn set_queue(&mut self,v:&Value)->Result<(),String>{
 let ids:Vec<String>=if let Some(id)=v["playlist"].as_str(){self.index.playlists.iter().find(|p|p.id==id).ok_or("Playlist not found")?.tracks.clone()}else if let Some(folder)=v["folder"].as_str(){self.index.tracks.iter().filter(|t|t.folder==folder).map(|t|t.id.clone()).collect()}else{self.index.tracks.iter().map(|t|t.id.clone()).collect()};
 if ids.is_empty(){return Err("No tracks in selection".into())}if let Some(id)=v["id"].as_str(){if !ids.iter().any(|x|x==id){return Err("Track outside selection".into())}}self.queue=ids;Ok(())
 }
 pub fn next(&mut self,back:bool,automatic:bool)->Option<String>{let ids:Vec<String>=if self.queue.is_empty(){self.index.tracks.iter().map(|t|t.id.clone()).collect()}else{self.queue.iter().filter(|id|self.index.tracks.iter().any(|t|&t.id==*id)).cloned().collect()};let n=ids.len();if n==0{return None}let i=self.current.as_ref().and_then(|id|ids.iter().position(|t|t==id)).unwrap_or(0);let j=if self.shuffle&&n>1{let mut b=[0;8];fs::File::open("/dev/urandom").ok()?.read_exact(&mut b).ok()?;(i+1+u64::from_ne_bytes(b) as usize%(n-1))%n}else if back{(i+n-1)%n}else if i+1==n&&automatic&&!self.repeat{return None}else{(i+1)%n};self.select(&ids[j]).ok()}
 pub fn groups(&self)->Value{json!({"folders":self.index.folders,"playlists":self.index.playlists})}
 pub fn organize(&mut self,v:&Value)->Result<(),String>{self.check()?;let old=self.index.clone();let result=self.organize_inner(v);if result.is_err(){self.index=old;}result}
 fn organize_inner(&mut self,v:&Value)->Result<(),String>{
 let action=v["action"].as_str().unwrap_or("");let id=v["id"].as_str().unwrap_or("");
 let name=||->Result<String,String>{let s=v["name"].as_str().unwrap_or("").trim();if s.is_empty()||s.chars().count()>80||s.chars().any(char::is_control){Err("Name: 1..80 characters".into())}else{Ok(s.into())}};
 let folder_ok=|id:&str|id.is_empty()||self.index.folders.iter().any(|f|f.id==id);
 match action{
 "music_network_add"=>{let u=v["url"].as_str().ok_or("Missing URL")?;let folder=v["folder"].as_str().unwrap_or("");if !super::valid_url(u)||!url::Url::parse(u).is_ok_and(|x|x.path().to_lowercase().ends_with(".mp3"))||!folder_ok(folder)||self.index.tracks.len()>=500{return Err("Use a direct HTTP(S) MP3 URL and valid folder".into())}self.index.tracks.push(Track{id:random_id()?,title:name()?,artist:String::new(),album:String::new(),bytes:0,source:u.into(),folder:folder.into()});},
 "music_folder_create"=>{let parent=v["parent"].as_str().unwrap_or("");if !folder_ok(parent)||self.index.folders.len()>=100{return Err("Invalid parent or folder limit".into())}self.index.folders.push(Folder{id:random_id()?,name:name()?,parent:parent.into()});},
 "music_folder_rename"=>{let name=name()?;self.index.folders.iter_mut().find(|f|f.id==id).ok_or("Folder not found")?.name=name;},
 "music_folder_delete"=>{if self.upload.as_ref().is_some_and(|u|u.folder==id)||self.index.folders.iter().any(|f|f.parent==id)||self.index.tracks.iter().any(|t|t.folder==id){return Err("Folder must be empty".into())}let i=self.index.folders.iter().position(|f|f.id==id).ok_or("Folder not found")?;self.index.folders.remove(i);},
 "music_move"=>{let folder=v["folder"].as_str().unwrap_or("");if !folder_ok(folder){return Err("Folder not found".into())}self.index.tracks.iter_mut().find(|t|t.id==id).ok_or("Track not found")?.folder=folder.into();},
 "music_playlist_save"=>{let name=name()?;let tracks:Vec<String>=serde_json::from_value(v["tracks"].clone()).map_err(|_|"Invalid playlist")?;if tracks.iter().collect::<std::collections::HashSet<_>>().len()!=tracks.len(){return Err("Each track can appear once per playlist".into())}if tracks.len()>500||tracks.iter().any(|id|!self.index.tracks.iter().any(|t|&t.id==id)){return Err("Unknown track or playlist too long".into())}if id.is_empty(){if self.index.playlists.len()>=100{return Err("Playlist limit".into())}self.index.playlists.push(Playlist{id:random_id()?,name,tracks});}else{let p=self.index.playlists.iter_mut().find(|p|p.id==id).ok_or("Playlist not found")?;p.name=name;p.tracks=tracks;}},
 "music_playlist_delete"=>{let i=self.index.playlists.iter().position(|p|p.id==id).ok_or("Playlist not found")?;self.index.playlists.remove(i);},_=>return Err("Unknown library action".into())}self.save()
 }
 pub fn options(&mut self,repeat:bool,shuffle:bool)->Result<(),String>{self.check()?;let old=(self.index.repeat,self.index.shuffle);self.index.repeat=repeat;self.index.shuffle=shuffle;if let Err(e)=self.save(){self.index.repeat=old.0;self.index.shuffle=old.1;return Err(e)}self.repeat=repeat;self.shuffle=shuffle;Ok(())}
 fn used(&self)->u64{fs::read_dir(&self.root).ok().into_iter().flatten().flatten().filter(|e|e.path().extension().is_some_and(|s|s=="mp3")).filter_map(|e|e.metadata().ok()).map(|m|m.len()).sum()}
 fn free(&self)->u64{use std::os::unix::ffi::OsStrExt;let Ok(p)=std::ffi::CString::new(self.root.as_os_str().as_bytes())else{return 0};unsafe{let mut s:libc::statvfs=std::mem::zeroed();if libc::statvfs(p.as_ptr(),&mut s)!=0{0}else{(s.f_bavail as u64).saturating_mul(s.f_frsize as u64)}}}
 fn save(&self)->Result<(),String>{let p=self.root.join("library.json.tmp");let b=serde_json::to_vec(&self.index).map_err(|e|e.to_string())?;let mut f=fs::File::create(&p).map_err(|e|e.to_string())?;f.write_all(&b).and_then(|_|f.sync_all()).and_then(|_|fs::rename(p,self.root.join("library.json"))).map_err(|e|e.to_string())}
 pub fn list(&self)->Value{json!({"tracks":self.index.tracks,"folders":self.index.folders,"playlists":self.index.playlists,"used":self.used(),"limit":LIMIT,"free":self.free(),"reserve":RESERVE,"error":self.error})}
 pub fn brief(&self)->Value{json!(self.index.tracks.iter().map(|t|json!({"id":t.id,"title":t.title,"folder":t.folder})).collect::<Vec<_>>())}
 pub fn expire(&mut self){if self.upload.as_ref().is_some_and(|u|u.touched.elapsed().as_secs()>120){self.cancel();}}
 fn cancel(&mut self){if let Some(u)=self.upload.take(){let _=fs::remove_file(self.root.join(format!("{}.part",u.id)));}}
 pub fn upload_status(&self)->Value{self.upload.as_ref().map(|u|json!({"id":u.id,"offset":u.written,"size":u.size})).unwrap_or(Value::Null)}
 pub fn edit(&mut self,v:&Value)->Result<(),String>{self.check()?;self.expire();match v["action"].as_str().unwrap_or(""){
 "music_library"=>{},
 "music_upload_begin"=>{
  if self.upload.is_some(){return Err("Upload already active; cancel or wait two minutes".into())}
  let folder=v["folder"].as_str().unwrap_or("");if !folder.is_empty()&&!self.index.folders.iter().any(|f|f.id==folder){return Err("Folder not found".into())}let name=v["name"].as_str().ok_or("Missing filename")?;let size=v["size"].as_u64().filter(|n|*n>0&&*n<=128*1024*1024).ok_or("File must be 1 byte to 128 MiB")?;
  if !name.to_lowercase().ends_with(".mp3"){return Err("Only MP3 files are supported".into())}
  if self.index.tracks.len()>=500||self.used()+size>LIMIT||self.free()<size+RESERVE{return Err("Library limit or free-space reserve reached".into())}
  let mut b=[0;16];fs::File::open("/dev/urandom").and_then(|mut f|f.read_exact(&mut b)).map_err(|e|e.to_string())?;let id=b.iter().map(|v|format!("{v:02x}")).collect::<String>();use std::os::unix::fs::OpenOptionsExt;fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(self.root.join(format!("{id}.part"))).map_err(|e|e.to_string())?;
  let title=name[..name.len()-4].chars().filter(|c|!c.is_control()).take(120).collect::<String>();self.upload=Some(Upload{folder:folder.into(),id,title:if title.trim().is_empty(){"MP3".into()}else{title},size,written:0,touched:Instant::now()});
 },
 "music_upload_chunk"=>{
  let free=self.free();let u=self.upload.as_mut().ok_or("No upload")?;if v["id"].as_str()!=Some(&u.id)||v["offset"].as_u64()!=Some(u.written){return Err("Upload ID or offset mismatch".into())}
  let h=v["hex"].as_str().ok_or("Missing chunk")?;if h.is_empty()||h.len()>49152||h.len()%2!=0{return Err("Invalid chunk size".into())}
  let b=h.as_bytes().chunks_exact(2).map(|c|{let s=std::str::from_utf8(c).map_err(|_|"Invalid hex")?;u8::from_str_radix(s,16).map_err(|_|"Invalid hex")}).collect::<Result<Vec<_>,_>>()?;
  if u.written+b.len() as u64>u.size||free<RESERVE+b.len() as u64{return Err("Size or free-space limit reached".into())}
  let p=self.root.join(format!("{}.part",u.id));if let Err(e)=fs::OpenOptions::new().append(true).open(p).and_then(|mut f|f.write_all(&b)){self.cancel();return Err(e.to_string())}u.written+=b.len() as u64;u.touched=Instant::now();
 },
 "music_upload_cancel"=>{if self.upload.as_ref().is_some_and(|u|v["id"].as_str()!=Some(&u.id)){return Err("Upload ID mismatch".into())}self.cancel();},
 "music_upload_finish"=>{
  let u=self.upload.as_ref().ok_or("No upload")?;if v["id"].as_str()!=Some(&u.id)||u.size!=u.written{return Err("Incomplete upload".into())}
  let p=self.root.join(format!("{}.part",u.id));let mut head=[0;10];let n=fs::File::open(&p).and_then(|mut f|f.read(&mut head)).map_err(|e|e.to_string())?;
  if n<3||!(head.starts_with(b"ID3")||(head[0]==255&&head[1]&0xe0==0xe0)){self.cancel();return Err("No MP3/ID3 header".into())}
  let track=Track{source:String::new(),folder:u.folder.clone(),id:u.id.clone(),title:u.title.clone(),artist:String::new(),album:String::new(),bytes:u.size};let target=self.path(&u.id);fs::File::open(&p).and_then(|f|f.sync_all()).and_then(|_|fs::rename(&p,&target)).map_err(|e|e.to_string())?;
  self.index.tracks.push(track);if let Err(e)=self.save(){self.index.tracks.pop();let _=fs::rename(target,p);return Err(e)}self.upload=None;
 },
 "music_edit"=>{
  let id=v["id"].as_str().ok_or("Missing ID")?;let i=self.index.tracks.iter().position(|t|t.id==id).ok_or("Track not found")?;let old=self.index.tracks[i].clone();
  for k in ["title","artist","album"]{let s=v[k].as_str().ok_or("Missing metadata")?;if s.chars().count()>120||s.chars().any(char::is_control)||(k=="title"&&s.trim().is_empty()){return Err("Metadata: max 120 characters, title required".into())}}
  let t=&mut self.index.tracks[i];t.title=v["title"].as_str().unwrap().trim().into();t.artist=v["artist"].as_str().unwrap().trim().into();t.album=v["album"].as_str().unwrap().trim().into();if let Err(e)=self.save(){self.index.tracks[i]=old;return Err(e)}
 },
 "music_delete"=>{
  let id=v["id"].as_str().ok_or("Missing ID")?;if self.current.as_deref()==Some(id){return Err("Switch to Radio before deleting the selected track".into())}
  let i=self.index.tracks.iter().position(|t|t.id==id).ok_or("Track not found")?;let remote=!self.index.tracks[i].source.is_empty();let file=self.path(id);let trash=self.root.join(format!("{id}.deleted"));if !remote{fs::rename(&file,&trash).map_err(|e|e.to_string())?;}
  let old=self.index.clone();self.index.tracks.remove(i);for p in &mut self.index.playlists{p.tracks.retain(|x|x!=id);}if let Err(e)=self.save(){self.index=old;let _=fs::rename(trash,file);return Err(e)}if !remote{fs::remove_file(trash).map_err(|e|e.to_string())?;}
 },_=>return Err("Unknown library action".into())}Ok(())}
}

fn random_id()->Result<String,String>{let mut b=[0;16];fs::File::open("/dev/urandom").and_then(|mut f|f.read_exact(&mut b)).map_err(|e|e.to_string())?;Ok(b.iter().map(|v|format!("{v:02x}")).collect())}
