use bmc_wasm_sdk::*;
pub fn draw(label:&str,x:f32,y:f32,w:f32,h:f32,color:Color)->Option<Draw>{let (icon,ratio)=match label{
"再生"=>(include_svg!("assets/jp/play.svg"),1.6f32),
"停止"=>(include_svg!("assets/jp/stop.svg"),1.6f32),
"消音"=>(include_svg!("assets/jp/mute.svg"),1.6f32),
"消音解除"=>(include_svg!("assets/jp/unmute.svg"),3.2f32),
"再生中"=>(include_svg!("assets/jp/playing.svg"),2.4f32),
"接続中"=>(include_svg!("assets/jp/connecting.svg"),2.4f32),
"再接続中"=>(include_svg!("assets/jp/retrying.svg"),3.2f32),
"消音中"=>(include_svg!("assets/jp/muted.svg"),2.4f32),
"通知"=>(include_svg!("assets/jp/alarm.svg"),1.6f32),
"停止中"=>(include_svg!("assets/jp/stopped.svg"),2.4f32),
"サービス待機中"=>(include_svg!("assets/jp/waiting.svg"),5.6f32),
"サービスに接続できません"=>(include_svg!("assets/jp/error.svg"),9.6f32),
"未登録"=>(include_svg!("assets/jp/empty.svg"),2.4f32),
"プリセット"=>(include_svg!("assets/jp/presets.svg"),4.0f32),
"音量"=>(include_svg!("assets/jp/volume.svg"),1.6f32),
_=>return None,};let dh=h.min(w/ratio);let dw=dh*ratio;Some(Draw::svg_id(x+(w-dw)/2.0,y+(h-dh)/2.0,dw,dh,ensure_registered(&icon),color))}