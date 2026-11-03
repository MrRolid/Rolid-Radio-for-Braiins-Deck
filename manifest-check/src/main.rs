use bmc_widget_manifest::Manifest;
fn main() {
 let path=std::env::args().nth(1).expect("Usage: rolid-manifest-check manifest.json");
 match Manifest::from_reader(std::fs::File::open(&path).expect("read manifest")) {
  Ok(_)=>println!("Deck manifest parser: OK ({path})"),
  Err(e)=>{eprintln!("Deck manifest parser: {e}");std::process::exit(1);}
 }
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn release_manifest_passes_deck_parser() {
  let path=format!("{}/../manifest.json",env!("CARGO_MANIFEST_DIR"));Manifest::from_reader(std::fs::File::open(path).unwrap()).unwrap();
 }
}
