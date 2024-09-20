pub use chunks::exta::offscreen::{BlockData, BlockDataChunk, ExtaOffscreen};
pub use chunks::exta::ClipExtaHeader;
pub use chunks::head::ClipHeader;
pub use chunks::sqli::{ClipDb, ClipSqliteChunk};

mod dbutil;
mod chunks;

pub mod util {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::unix::fs::FileExt;
    use std::path::Path;

    use crate::{ExtaOffscreen, ClipDb, ClipExtaHeader, ClipHeader, ClipSqliteChunk};
    use crate::dbutil::BorrowedConnection;

    // panics for everything bc they're just utils for figuring out how the format works

    pub fn with_clip_file(clip_file: &mut File, consumer: impl Fn(ClipHeader, &[u8], &mut File)) {
        clip_file.seek(SeekFrom::Start(0)).unwrap();
        let mut header_buf: [u8; 512] = [0u8; 512];

        clip_file.read_exact(&mut header_buf).unwrap();

        let (_, header) = ClipHeader::parse(&header_buf).unwrap();

        let mut sql_buf = vec![];
        clip_file.seek(SeekFrom::Start(header.sqlite_chunk_pos)).unwrap();
        clip_file.read_to_end(&mut sql_buf).expect("Failed to read SQLite chunk");

        let (_, data) = ClipSqliteChunk::extract_data(&sql_buf).expect("Failed to parse SQLite chunk");

        consumer(header, data, clip_file);
        clip_file.seek(SeekFrom::Start(0)).unwrap();
    }

    pub fn export_clip_sqlite(clip_file: &mut File, sql_out: &Path) {
        with_clip_file(clip_file, |_, sqlite_data, _| {
            let mut out = File::create_new(sql_out).unwrap();
            out.write_all(sqlite_data).unwrap();
        })
    }

    pub fn export_offscreen_for_rasters(clip_file: &mut File, out_dir: &Path) {
        with_clip_file(clip_file, |_, sqlite_data, file| {
            let c = BorrowedConnection::from(sqlite_data).expect("Unable to open sqlite");
            let db = ClipDb::with_conn(&c.conn);

            let layer_ids = db.get_layer_ids_for_canvas(1);

            for id in layer_ids {
                let offsets = db.get_offscreen_exta_offsets(id);
                if offsets.is_empty() { continue }

                for offset in offsets {
                    println!("Offset: {offset}");

                    let mut buf = [0u8; 512];
                    file.read_exact_at(&mut buf, offset as _).unwrap();

                    let (rem, exta) = ClipExtaHeader::parse(&buf).unwrap();

                    let mut exta_buf = vec![0u8; exta.body_size as _];

                    let body_offset = offset as usize + (512 - rem.len());
                    file.read_exact_at(&mut exta_buf, body_offset as u64).unwrap();

                    let (_, block): (&[u8], ExtaOffscreen) = ExtaOffscreen::parse(exta_buf.as_slice()).expect("Failed block parse");

                    for (i, chunk) in block.chunks.iter().enumerate() {
                        let mut data = chunk.decompress();
                        
                        // dir
                        let dir = out_dir.join(format!("layer-id_{id}/chunk-offset_{offset}"));
                        std::fs::create_dir_all(&dir).unwrap();
                        
                        // blocks
                        let out = File::create_new(&dir.join(format!("block_{i:0>5}")));
                        out.unwrap().write_all(&mut data).unwrap();
                    }
                }
            }
        })
    }
}
