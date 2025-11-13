use rusqlite::Row;
use std::io::Cursor;

use crate::sql::ClipDb;
use binrw::{binread, binwrite, BinRead};
use num_enum::TryFromPrimitive;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};

// a lot of these actually have smaller possible values
// we're just going with the largest that would fit for consistency

#[binread]
#[binwrite]
#[brw(big)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum FilterLayerInfo {
    // FilterLayerInfo
    // kind: u32
    // size: u32
    // data
    #[brw(magic = 1u32)]
    BrightnessContrast {
        #[br(temp, assert(size == 8))]
        #[bw(calc = 8)]
        size: u32,
        b: i32,
        c: i32,
    }, // 1

    #[brw(magic = 2u32)]
    LevelCorrection, // 2 todo

    #[brw(magic = 3u32)]
    ToneCurve, // 3 todo

    #[brw(magic = 4u32)]
    Hsl {
        #[br(temp, assert(size == 12))]
        #[bw(calc = 12)]
        size: u32,
        h: i32,
        s: i32,
        l: i32,
    }, // 4

    #[brw(magic = 5u32)]
    ColorBalance, // 5 todo

    #[brw(magic = 6u32)]
    ReverseGradient, // 6

    #[brw(magic = 7u32)]
    Posterization {
        #[br(temp, assert(size == 4))]
        #[bw(calc = 4)]
        size: u32,
        val: u32,
    }, // 7

    #[brw(magic = 8u32)]
    Binarization {
        #[br(temp, assert(size == 4))]
        #[bw(calc = 4)]
        size: u32,
        val: u32,
    }, // 8

    #[brw(magic = 9u32)]
    GradientMap, // 9 todo

    Unknown(u32),
}

impl FromSql for FilterLayerInfo {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let v = Self::read(&mut Cursor::new(value.as_bytes()?));
        v.map_err(|_| FromSqlError::InvalidType)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, TryFromPrimitive)]
#[repr(i64)]
pub enum LayerBlendMode {
    Normal = 0,
    Darken = 1,
    Multiply = 2,
    ColorBurn = 3,
    LinearBurn = 4,
    Subtract = 5,
    DarkerColor = 6,
    Lighten = 7,
    Screen = 8,
    ColorDodge = 9,
    GlowDodge = 10,
    Add = 11,
    AddGlow = 12,
    LighterColor = 13,
    Overlay = 14,
    SoftLight = 15,
    HardLight = 16,
    VisidLight = 17,
    LinearLight = 18,
    PinLight = 19,
    HardMix = 20,
    Difference = 21,
    Exclusion = 22,
    Hue = 23,
    Saturation = 24,
    Color = 25,
    Brightness = 26,
    Divide = 36,
    #[num_enum(catch_all)]
    Unknown(i64),
}

impl FromSql for LayerBlendMode {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        LayerBlendMode::try_from(value.as_i64()?).map_err(|_| FromSqlError::InvalidType)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, TryFromPrimitive)]
#[repr(i64)]
pub enum LayerKind {
    Dummy = 256,  // root folder
    Paper = 1584, // paper layer, only one per canvas

    // vector, folder, 3d, folder, frame folder, gradient, fill, tone
    Other = 0,
    OtherMasked = 2,

    // raster
    Raster = 1,
    RasterMasked = 3,

    // filter
    Filter = 4096,
    FilterMasked = 4098,

    #[num_enum(catch_all)]
    Unknown(i64),
}

impl FromSql for LayerKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        LayerKind::try_from(value.as_i64()?).map_err(|_| FromSqlError::InvalidType)
    }
}

// todo: needs more decoding
pub struct Layer {
    pub id: i64,
    pub canvas_id: i64,
    pub name: String,
    pub kind: LayerKind,

    pub opacity: i64, // this actually only goes to 256
    pub composite: LayerBlendMode,
}

impl Layer {
    fn from_row(r: &Row) -> Result<Layer, rusqlite::Error> {
        Ok(Layer {
            id: r.get("MainId")?,
            canvas_id: r.get("CanvasId")?,
            name: r.get("LayerName")?,
            kind: r.get("LayerType")?,
            opacity: r.get("LayerOpacity")?,
            composite: r.get("LayerComposite")?,
        })
    }
}

impl<'a> ClipDb<'a> {
    fn get_ext_id_offsets_for_layer(
        &self,
        table_name: &str,
        ext_id_colum_name: &str,
        layer_id: i64,
    ) -> Result<Vec<i64>, rusqlite::Error> {
        if !self.table_exists(table_name) {
            return Ok(vec![]);
        }

        // i give up- im bored with sql
        let stmt = self.conn.prepare_cached(&format!(
            "select ExternalChunk.Offset from ExternalChunk \
            inner join {table_name} on hex(ExternalChunk.ExternalID) = hex({table_name}.{ext_id_colum_name}) \
            where {table_name}.LayerId = ?1"
        ));

        stmt?.query_map([layer_id], |r| r.get(0))?.collect()
    }

    pub fn get_offscreen_exta_offsets(&self, layer_id: i64) -> Result<Vec<i64>, rusqlite::Error> {
        self.get_ext_id_offsets_for_layer("Offscreen", "BlockData", layer_id)
    }

    pub fn get_offscreen_vector_offsets(&self, layer_id: i64) -> Result<Vec<i64>, rusqlite::Error> {
        self.get_ext_id_offsets_for_layer("VectorObjectList", "VectorData", layer_id)
    }

    /// gets layers in the canvas with the given canvas ID
    pub fn get_layer_ids_for_canvas(&self, canvas_id: i64) -> Result<Vec<i64>, rusqlite::Error> {
        let stmt = self
            .conn
            .prepare_cached("SELECT MainId FROM Layer WHERE CanvasId=?1");

        stmt?.query_map([canvas_id], |r| r.get(0))?.collect()
    }

    /// gets the layer for the given ID
    pub fn get_layer(&self, layer_id: i64) -> Result<Layer, rusqlite::Error> {
        let stmt = self
            .conn()
            .prepare_cached("SELECT * FROM Layer WHERE MainId=?1");

        stmt?.query_row([layer_id], Layer::from_row)
    }

    pub fn get_fiter_layer_info(&self, layer_id: i64) -> Result<FilterLayerInfo, rusqlite::Error> {
        let stmt = self
            .conn()
            .prepare_cached("SELECT FilterLayerInfo FROM Layer WHERE MainId=?1");
        stmt?.query_row([layer_id], |r| r.get(0))
    }
}
