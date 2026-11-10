use std::{fs,io::{Write},os::unix::fs::OpenOptionsExt};
use serde::{Serialize,Deserialize};use serde_json::{Value,json};
#[derive(Serialize,Deserialize)]struct Access{enabled:bool,key:String}
fn load()->Result<Access,String>{let p=super::data_dir();if let Ok(b)=fs::read(p.join("web-access.json")){return serde_json::from_slice(&b).map_err(|e|e.to_string())}Ok(Access{enabled:false,key:String::new()})}
pub fn init()->Result<(),String>{load().map(|_|())}
pub fn info()->Result<Value,String>{let a=load()?;Ok(json!({"access":{"enabled":a.enabled,"key":a.key}}))}
pub fn enabled()->bool{load().map(|a|a.enabled).unwrap_or(true)}
pub fn authorized(auth:&str)->bool{load().is_ok_and(|a|!a.enabled||equal(auth,&format!("Bearer {}",a.key)))}
fn equal(a:&str,b:&str)->bool{a.len()==b.len()&&a.bytes().zip(b.bytes()).fold(0,|x,(a,b)|x|(a^b))==0}
pub fn update(v:&Value)->Result<(),String>{let mut a=load()?;a.enabled=v["enabled"].as_bool().ok_or("Enabled must be boolean")?;let key=v["key"].as_str().unwrap_or("");if v["key"].is_string(){if key.len()>128||key.trim()!=key||!key.bytes().all(|b|b.is_ascii_graphic()||b==b' '){return Err("Password: up to 128 printable ASCII characters, no leading/trailing spaces".into())}a.key=key.into();}if a.key.is_empty(){a.enabled=false;}let p=super::data_dir().join("web-access.json.tmp");let mut f=fs::OpenOptions::new().write(true).truncate(true).create(true).mode(0o600).open(&p).map_err(|e|e.to_string())?;f.write_all(&serde_json::to_vec(&a).unwrap()).and_then(|_|f.sync_all()).and_then(|_|fs::rename(p,super::data_dir().join("web-access.json"))).map_err(|e|e.to_string())?;Ok(())}
