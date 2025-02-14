use std::fs::File;
use clipdecode::util::{export_clip_sqlite, export_offscreen_for_rasters};

fn main() {
    let mut f = File::open("tmp/4pt.clip").unwrap();
    export_clip_sqlite(&mut f, "tmp/4pt/4pt.sqlite".as_ref());
    export_offscreen_for_rasters(&mut f, "tmp/4pt/offsc".as_ref())
}
