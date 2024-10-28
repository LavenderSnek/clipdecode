use std::fs::File;
use clipdecode::util::{export_clip_sqlite, export_offscreen_for_rasters};

fn main() {
    let mut f = File::open("assets/plainpainting.clip").unwrap();
    export_clip_sqlite(&mut f, "tmp/plain.sqlite".as_ref());
    export_offscreen_for_rasters(&mut f, "tmp/offscreen".as_ref())
}
