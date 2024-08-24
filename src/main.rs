use std::fs::File;
use std::path::Path;
use clipdecode::util;



fn mim() {
    let s = "/Users/snek/code/cspdecoding/oddshape.clip";
    let mut d = File::open(Path::new(&s)).unwrap();
    
    util::export_offscreen_for_rasters(&mut d, Path::new("oddshape-decomp"));
}

fn main() {
    mim()
}
