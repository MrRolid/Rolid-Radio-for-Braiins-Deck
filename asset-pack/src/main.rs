use std::{path::Path,fs};
use bmc_wasm_assets::{contains_package_asset_section,extract_package_assets};
fn main(){
 let a:Vec<String>=std::env::args().collect();
 if a.len()==3 && a[1]=="verify" {assert!(!contains_package_asset_section(&fs::read(&a[2]).unwrap()).unwrap(),"unstripped package assets");return;}
 assert_eq!(a.len(),5,"Usage: asset-pack INPUT OUTPUT ASSETS ARTIFACT_ROOT");
 extract_package_assets(Path::new(&a[1]),Some(Path::new(&a[4])),Path::new(&a[2]),Path::new(&a[3])).unwrap();
 assert!(!contains_package_asset_section(&fs::read(&a[2]).unwrap()).unwrap());
}
