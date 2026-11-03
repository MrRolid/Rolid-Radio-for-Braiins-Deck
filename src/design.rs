use bmc_wasm_sdk::*;
use crate::jp;
pub const BG:u32=0x101518;pub const PANEL:u32=0x20282c;pub const LINE:u32=0x3b474b;pub const INK:u32=0xe7e8de;pub const MUTED:u32=0x9aa8a9;pub const AMBER:u32=0xe9b960;
fn c(v:u32)->Color{Color::from_hex(v)}
pub fn label(t:&str,x:f32,y:f32,w:f32,h:f32,size:u32,color:u32)->Draw{
 if let Some(d)=jp::draw(t,x,y,w,h.min(size as f32*1.3),c(color)){return d;}
 Draw::autofit_text(x,y,w,h,t,TextStyle{size,align:TextAlign::Center,vertical_align:VerticalAlign::Center,color:c(color),..TextStyle::default()})
}
pub fn key(id:&str,t:&str,sub:&str,w:f32,h:f32,active:bool,disabled:bool)->Node{
 let fill=if active{0x353225}else{PANEL};let edge=if active{AMBER}else{LINE};
 let mut draws=vec![Draw::rect(0.0,0.0,w,h,c(edge)),Draw::rect(1.0,1.0,w-2.0,h-2.0,c(fill))];
 if active{draws.push(Draw::rect(8.0,4.0,w-16.0,2.0,c(AMBER)));}
 let color=if disabled{0x647174}else if active{AMBER}else{INK};
 if t=="@list"||t=="@settings" {for i in 0..3{let y=h/2.0-10.0+i as f32*10.0;draws.push(Draw::rect(w/2.0-12.0,y,24.0,2.0,c(color)));if t=="@settings"{draws.push(Draw::rect(w/2.0-8.0+(i%2) as f32*12.0,y-3.0,4.0,8.0,c(color)));}}}else if t=="-" || t=="+" {draws.push(Draw::rect(w/2.0-10.0,h/2.0-1.0,20.0,2.0,c(color)));if t=="+" {draws.push(Draw::rect(w/2.0-1.0,h/2.0-10.0,2.0,20.0,c(color)));}}else if sub.is_empty(){draws.push(label(t,5.0,(h-30.0)/2.0,w-10.0,30.0,24,color));}else{
 draws.push(label(t,4.0,7.0,w-8.0,h*0.48,23,color));draws.push(label(sub,3.0,h*0.64,w-6.0,h*0.25,11,color));}
 let props=props!(width:w,height:h);
 if disabled{canvas(props,draws)}else{touchable(id,props,draws)}
}
#[derive(Clone,Copy)] pub struct Layout {pub width:f32,pub head:f32,pub preset:f32,pub transport:f32,pub volume:f32}
pub fn layout(w:u32,h:u32)->Layout{let available=h as f32-24.0;let scale=(available/214.0).max(1.0);Layout{width:w as f32-12.0,head:58.0*scale,preset:48.0*scale,transport:56.0*scale,volume:52.0*scale}}
