/// Measure stereo signed 16-bit little-endian PCM. Values are 0..1000 linear amplitude.
pub fn levels(pcm:&[u8])->([u64;2],[u64;2]){
 let mut sum=[0f64;2];let mut peak=[0u64;2];let frames=pcm.len()/4;
 for frame in pcm.chunks_exact(4){for c in 0..2{let v=i16::from_le_bytes([frame[2*c],frame[2*c+1]]) as i32;let a=v.unsigned_abs() as u64;sum[c]+=(a*a) as f64;peak[c]=peak[c].max(a);}}
 if frames==0{return ([0;2],[0;2])}
 ([((sum[0]/frames as f64).sqrt()*1000.0/32768.0) as u64,((sum[1]/frames as f64).sqrt()*1000.0/32768.0) as u64],[peak[0]*1000/32768,peak[1]*1000/32768])
}
#[cfg(test)]mod tests{use super::*;#[test]fn known_pcm(){assert_eq!(levels(&[0;40]),([0,0],[0,0]));let data=[0,64,0,128].repeat(100);assert_eq!(levels(&data),([500,1000],[500,1000]));}}
