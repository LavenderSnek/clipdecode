use binrw::BinRead;
use clipdecode::chunk::*;
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
};

fn export_sqlite<R: Read + Seek, W: Write>(src: &mut R, dst: &mut W) {
    let head_pos = CsfChunk::read(src).unwrap().head_chunk_pos;

    src.seek(SeekFrom::Start(head_pos)).unwrap();
    let sqlite_pos = HeadChunk::read(src).unwrap().sqlite_chunk_pos;

    src.seek(SeekFrom::Start(sqlite_pos)).unwrap();
    let size = SqliteChunkHeader::read(src).unwrap().body_size;

    let mut bytes = src.take(size);
    io::copy(&mut bytes, dst).unwrap();
}

fn main() {
    let mut src = File::open("/Users/snek/code/clipdecode/tmp/randomshit.clip").unwrap();
    let mut dst = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("tmp/randomshit.sqlite")
        .unwrap();
    export_sqlite(&mut src, &mut dst);
}
