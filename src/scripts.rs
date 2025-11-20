use crate::chunk::*;
use crate::exta::offscreen::ExtaOffscreen;
use crate::sql::external::{OffscreenMeta, VectorObjListMeta};
use crate::sql::{ClipDb, LayerId};
use binrw::{BinRead, BinWrite};
use flate2::read::ZlibDecoder;
use rusqlite::Connection;
use std::io::{Read, Seek, SeekFrom, Write};
use std::{error, fs, io};

fn into_io_err<E>(e: E) -> io::Error
where
    E: Into<Box<dyn error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::InvalidData, e)
}

pub fn export_sqlite<R: Read + Seek, W: Write>(src: &mut R, dst: &mut W) -> std::io::Result<()> {
    let head_pos = CsfChunk::read(src).map_err(into_io_err)?.head_chunk_pos;

    src.seek(SeekFrom::Start(head_pos))?;

    let sqlite_pos = HeadChunk::read(src).map_err(into_io_err)?.sqlite_chunk_pos;

    src.seek(SeekFrom::Start(sqlite_pos))?;
    let size = SqliteChunkHeader::read(src).map_err(into_io_err)?.body_size;

    let mut bytes = src.take(size);
    io::copy(&mut bytes, dst)?;
    Ok(())
}

pub fn splat_layer_exta<R: Read + Seek, P: AsRef<std::path::Path>>(
    db: &ClipDb,
    layer: LayerId,
    clip: &mut R,
    out_dir: P,
) -> std::io::Result<()> {
    fs::create_dir_all(&out_dir)?;

    // i could do this as a join in sql but thats too much abstraction for now
    let vecs: Vec<VectorObjListMeta> = db
        .get_vector_obj_list_ids_for_layer(layer)
        .map_err(into_io_err)?
        .into_iter()
        .map(|id| db.get_vector_obj_list_meta(id).map_err(into_io_err))
        .collect::<io::Result<_>>()?;

    for v in vecs {
        let offset = db
            .get_exta_chunk_offset(v.exta_id.clone())
            .map_err(into_io_err)?;
        clip.seek(SeekFrom::Start(offset as _))?;

        let hdr = ExtaChunkHeader::read(clip).map_err(into_io_err)?;

        let mut bytes = clip.take(hdr.body_size);

        let mut f = fs::File::create(out_dir.as_ref().join(format!("{}.vec.bin", v.exta_id.0)))?;
        io::copy(&mut bytes, &mut f)?;
    }

    let offsc: Vec<OffscreenMeta> = db
        .get_offscreen_ids_for_layer(layer)
        .map_err(into_io_err)?
        .into_iter()
        .map(|id| db.get_offscreen_meta(id).map_err(into_io_err))
        .collect::<io::Result<_>>()?;

    for o in offsc {
        let mut attr = fs::File::create(
            out_dir
                .as_ref()
                .join(format!("{}.offsc_attr.bin", o.exta_id.0)),
        )?;

        o.attribute.write(&mut attr).map_err(into_io_err)?;

        let exta_off = db.get_exta_chunk_offset(o.exta_id.clone());
        let Ok(offset) = exta_off else {
            continue;
        };

        clip.seek(SeekFrom::Start(offset as _))?;

        let _ = ExtaChunkHeader::read(clip).map_err(into_io_err)?;
        let body = ExtaOffscreen::read(clip).map_err(into_io_err)?;

        for (i, b) in body.blocks.into_iter().enumerate() {
            let Some(data) = b.data else {
                continue;
            };

            let p = out_dir
                .as_ref()
                .join(format!("{}.offsc_{}.bin", o.exta_id.0, i));

            let mut dec = ZlibDecoder::new(data.compressed_data.as_slice());
            let mut buffer = Vec::new();
            dec.read_to_end(&mut buffer)?;

            fs::write(p, buffer)?;
        }
    }

    Ok(())
}

pub fn splat_clip_file<R: Read + Seek, P: AsRef<std::path::Path>>(
    clip: &mut R,
    out_dir: P,
) -> std::io::Result<()> {
    let dir = out_dir.as_ref();
    fs::create_dir_all(dir)?;

    // export db
    let sqlite_path = dir.join("db.sqlite");
    let mut sqlite = fs::File::create(&sqlite_path)?;
    export_sqlite(clip, &mut sqlite)?;

    let conn = Connection::open(sqlite_path).map_err(into_io_err)?;
    let db = ClipDb::with_conn(&conn);

    let canvases = db.get_canvas_ids().map_err(into_io_err)?;
    for c in canvases {
        let p = db.get_preview_for_canvas(c).map_err(into_io_err)?;
        let img = p.image_data;
        fs::write(dir.join(format!("preview_canvas_{}.png", c.0)), img)?;

        let layers = db.get_layer_ids_for_canvas(c).map_err(into_io_err)?;

        for l in layers {
            splat_layer_exta(&db, l, clip, dir.join(format!("c{}_l{}", c.0, l.0)))?;
        }
    }

    Ok(())
}
