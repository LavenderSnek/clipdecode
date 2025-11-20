use std::io::Cursor;

use binrw::{binread, binwrite, BinRead};
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ValueRef},
    Row,
};

use crate::{
    parse_utl::Utf16BeTag,
    sql::{CanvasId, ClipDb, ExtaChunkId, LayerId, OffscreenId, VectorObjListId},
};

pub struct VectorObjListMeta {
    pub id: VectorObjListId,
    pub canvas_id: CanvasId,
    pub layer_id: LayerId,
    pub exta_id: ExtaChunkId,
}

impl VectorObjListMeta {
    fn from_row(r: &Row) -> Result<VectorObjListMeta, rusqlite::Error> {
        Ok(VectorObjListMeta {
            id: r.get("MainId")?,
            canvas_id: r.get("CanvasId")?,
            layer_id: r.get("LayerId")?,
            exta_id: r.get("VectorData")?,
        })
    }
}

#[binread]
#[binwrite]
#[brw(big)]
#[br(import(size: u32))]
#[derive(Debug)]
pub struct OffscreenParameter {
    #[brw(args("Parameter"))]
    tag: Utf16BeTag,

    #[br(count = size - tag.calc_size())]
    pub data: Vec<u8>,
}

#[binread]
#[binwrite]
#[brw(big)]
#[br(import(size: u32))]
#[derive(Debug)]
pub struct OffscreenInitColor {
    #[brw(args("InitColor"))]
    tag: Utf16BeTag,

    #[br(count = size - tag.calc_size())]
    pub data: Vec<u8>,
}

#[binread]
#[binwrite]
#[brw(big)]
#[br(import(size: u32))]
#[derive(Debug)]
pub struct OffscreenBlockSize {
    #[brw(args("BlockSize"))]
    tag: Utf16BeTag,

    #[br(count = size - tag.calc_size())]
    pub data: Vec<u8>,
}

#[binread]
#[binwrite]
#[brw(big)]
#[derive(Debug)]
pub struct OffscreenAttribute {
    #[br(temp, assert(_header_size == 16))]
    #[bw(calc = 16)]
    _header_size: u32,

    #[bw(calc = parameter.tag.calc_size() + parameter.data.len() as u32)]
    parameter_size: u32,

    #[bw(calc = initcolor.tag.calc_size() + initcolor.data.len() as u32)]
    initcolor_size: u32,

    #[bw(calc = blocksize.tag.calc_size() + blocksize.data.len() as u32)]
    blocksize_size: u32,

    #[br(args(parameter_size))]
    pub parameter: OffscreenParameter,

    #[br(args(initcolor_size))]
    pub initcolor: OffscreenInitColor,

    #[br(args(blocksize_size))]
    pub blocksize: OffscreenBlockSize,
}

impl FromSql for OffscreenAttribute {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let v = Self::read(&mut Cursor::new(value.as_bytes()?));
        v.map_err(|_| FromSqlError::InvalidType)
    }
}

pub struct OffscreenMeta {
    pub id: OffscreenId,
    pub canvas_id: CanvasId,
    pub layer_id: LayerId,
    pub attribute: OffscreenAttribute,
    pub exta_id: ExtaChunkId,
}

impl OffscreenMeta {
    fn from_row(r: &Row) -> Result<OffscreenMeta, rusqlite::Error> {
        Ok(OffscreenMeta {
            id: r.get("MainId")?,
            canvas_id: r.get("CanvasId")?,
            layer_id: r.get("LayerId")?,
            attribute: r.get("Attribute")?,
            exta_id: r.get("BlockData")?,
        })
    }
}

impl<'a> ClipDb<'a> {
    /// returns a list of all available canvas ids
    pub fn get_exta_offsets(&self) -> Result<Vec<i64>, rusqlite::Error> {
        let stmt = self
            .conn()
            .prepare_cached("SELECT Offset from ExternalChunk");
        stmt?.query_map([], |r| r.get(0))?.collect()
    }

    /// get external chunk offset for the given external id
    /// the chunk may not actually exist in the file
    pub fn get_exta_chunk_offset(&self, id: ExtaChunkId) -> Result<i64, rusqlite::Error> {
        let stmt = self
            .conn
            .prepare_cached("SELECT Offset FROM ExternalChunk WHERE ExternalID=?1");
        stmt.unwrap().query_row([id.0], |r| r.get(0))
    }

    pub fn get_offscreen_meta(&self, id: OffscreenId) -> Result<OffscreenMeta, rusqlite::Error> {
        let stmt = self
            .conn
            .prepare_cached("SELECT * FROM Offscreen WHERE MainId=?1");
        stmt.unwrap().query_row([id.0], OffscreenMeta::from_row)
    }

    pub fn get_vector_obj_list_meta(
        &self,
        id: VectorObjListId,
    ) -> Result<VectorObjListMeta, rusqlite::Error> {
        let stmt = self
            .conn
            .prepare_cached("SELECT * FROM VectorObjectList WHERE MainId=?1");
        stmt.unwrap().query_row([id.0], VectorObjListMeta::from_row)
    }
}
