use std::fs;

use clipdecode::scripts;

fn main() {
    let mut f = fs::File::open("tmp/cat.clip").unwrap();
    scripts::splat_clip_file(&mut f, "tmp/cat").unwrap();
}
