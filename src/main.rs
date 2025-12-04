use std::fs;

use clipdecode::scripts;

fn main() {
    let mut f = fs::File::open("tmp/grad.clip").unwrap();
    scripts::splat_clip_file(&mut f, "tmp/grad").unwrap();
}
