use std::{ffi::CStr,net::Ipv4Addr};
#[derive(Clone)]struct Address{name:String,ip:Ipv4Addr,flags:u32,wireless:bool}
fn choose(mut a:Vec<Address>)->Option<Address>{a.retain(|a|a.flags & libc::IFF_UP as u32 !=0 && a.flags & libc::IFF_LOOPBACK as u32==0 && !a.ip.is_loopback() && !a.ip.is_unspecified() && !a.ip.is_link_local() && !a.ip.is_multicast());a.sort_by_key(|a|(if a.wireless||a.name.starts_with("wl"){0}else if a.name.starts_with("eth")||a.name.starts_with("en")||a.name.starts_with("br-"){1}else{2},a.name.clone(),a.ip));a.into_iter().next()}
pub fn panel()->(Option<String>,Option<String>){
 let mut head=std::ptr::null_mut();if unsafe{libc::getifaddrs(&mut head)}!=0{return(None,None)}
 let mut p=head;let mut addresses=Vec::new();
 // The list remains valid until freeifaddrs. Read only AF_INET sockaddr_in entries.
 while !p.is_null(){unsafe{let a=&*p;if !a.ifa_addr.is_null()&&!a.ifa_name.is_null()&&(*a.ifa_addr).sa_family as i32==libc::AF_INET{
 let name=CStr::from_ptr(a.ifa_name).to_string_lossy().into_owned();let ip=Ipv4Addr::from((*(a.ifa_addr as *const libc::sockaddr_in)).sin_addr.s_addr.to_ne_bytes());let wireless=std::path::Path::new("/sys/class/net").join(&name).join("wireless").is_dir();addresses.push(Address{name,ip,flags:a.ifa_flags,wireless});}p=a.ifa_next;}}
 unsafe{libc::freeifaddrs(head)};choose(addresses).map(|a|(Some(format!("http://{}:18766",a.ip)),Some(a.name))).unwrap_or((None,None))
}
#[cfg(test)]mod tests{use super::*;fn a(n:&str,i:&str,up:bool)->Address{Address{name:n.into(),ip:i.parse().unwrap(),flags:if up{libc::IFF_UP as u32}else{0},wireless:false}}
#[test]fn prefer_wifi(){assert_eq!(choose(vec![a("eth0","192.0.2.3",true),a("wlan0","192.0.2.8",true)]).unwrap().name,"wlan0");}
#[test]fn ignore_unusable(){assert_eq!(choose(vec![a("wlan0","192.0.2.8",false),a("lo","127.0.0.1",true),a("eth0","192.0.2.3",true)]).unwrap().name,"eth0");assert!(choose(vec![a("wlan0","169.254.1.2",true)]).is_none());assert!(choose(vec![]).is_none());}
#[test]fn changed_ip(){assert_eq!(choose(vec![a("wlan0","192.0.2.9",true)]).unwrap().ip.to_string(),"192.0.2.9");}
}
