use std::{fs,io::{Read,Write},path::Path};
pub const ADDRESSES:[&str;4]=[
 "bc1qnpk42l7x6pexp0fw4a5j2tm3l5uerc6drpyw3l",
 "bc1q2hadjw5nwtyywhvd50ll0qelglgjeqn37988r9",
 "bc1qg24xphhjk05l7xaqfzr4w9nhgje39jm46xfjhv",
 "bc1qdyyp3p4wdde2nrtlqgjw84272nu6436h9yydat",
];
pub fn load_or_create(dir:&Path)->Result<String,String>{
 let path=dir.join("donation-address.txt");
 match fs::read_to_string(&path){
  Ok(a)=>return if ADDRESSES.contains(&a.trim()){Ok(a.trim().to_owned())}else{Err("Invalid saved donation address; not rerolling".into())},
  Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},
  Err(e)=>return Err(e.to_string()),
 }
 let mut byte=[0];fs::File::open("/dev/urandom").and_then(|mut f|f.read_exact(&mut byte)).map_err(|e|e.to_string())?;
 let address=ADDRESSES[(byte[0]&3) as usize];
 let tmp=dir.join("donation-address.txt.tmp");
 let mut f=fs::File::create(&tmp).map_err(|e|e.to_string())?;
 f.write_all(address.as_bytes()).and_then(|_|f.sync_all()).and_then(|_|fs::rename(&tmp,&path)).and_then(|_|fs::File::open(dir)?.sync_all()).map_err(|e|e.to_string())?;
 Ok(address.into())
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn persistence_and_corruption(){
  let dir=std::env::temp_dir().join(format!("rolid-donation-test-{}",std::process::id()));fs::create_dir_all(&dir).unwrap();
  let first=load_or_create(&dir).unwrap();assert!(ADDRESSES.contains(&first.as_str()));
  for _ in 0..8{assert_eq!(first,load_or_create(&dir).unwrap());}
  fs::write(dir.join("donation-address.txt"),"corrupt").unwrap();assert!(load_or_create(&dir).is_err());
  assert_eq!(fs::read_to_string(dir.join("donation-address.txt")).unwrap(),"corrupt");fs::remove_dir_all(dir).unwrap();
 }
}
