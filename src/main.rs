use std::fs;

use clipdecode::scripts;

fn main() {
    let mut f = fs::File::open("assets/colors.clip").unwrap();
    scripts::splat_clip_file(&mut f, "tmp/colors").unwrap();
}
