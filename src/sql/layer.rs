use rusqlite::Row;
use std::io::Cursor;

use crate::sql::{ClipDb, LayerId, MipmapId, OffscreenId, VectorObjListId};
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
    pub id: LayerId,
    pub canvas_id: i64,
    pub name: String,
    pub kind: LayerKind,

    pub opacity: i64, // only goes to 256
    pub composite: LayerBlendMode,
    pub clipped: bool,

    pub alpha_locked: bool,
    pub locked: bool,

    pub render_mipmap_id: MipmapId,
    pub mask_mipmap_id: MipmapId,

    // if (layerusepalletecolor) pallete color : option<color>
    pub filter_layer_info: Option<FilterLayerInfo>,
}

impl Layer {
    fn from_row(r: &Row) -> Result<Layer, rusqlite::Error> {
        let lock: u32 = r.get("LayerLock")?;

        Ok(Layer {
            id: r.get("MainId")?,
            canvas_id: r.get("CanvasId")?,
            name: r.get("LayerName")?,
            kind: r.get("LayerType")?,
            opacity: r.get("LayerOpacity")?,
            composite: r.get("LayerComposite")?,
            clipped: r.get("LayerClip")?,

            alpha_locked: lock & 16 != 0,
            locked: lock & 1 != 0,

            render_mipmap_id: r.get("LayerRenderMipmap")?,
            mask_mipmap_id: r.get("LayerLayerMaskMipmap")?,

            // todo: add check for table not existing vs parse error
            filter_layer_info: r.get("FilterLayerInfo").ok(), // might not exist
        })
    }
}

impl<'a> ClipDb<'a> {
    /// gets the layer for the given ID
    pub fn get_layer(&self, id: LayerId) -> Result<Layer, rusqlite::Error> {
        let stmt = self
            .conn()
            .prepare_cached("SELECT * FROM Layer WHERE MainId=?1");

        stmt?.query_row([id.0], Layer::from_row)
    }

    pub fn get_offscreen_ids_for_layer(
        &self,
        id: LayerId,
    ) -> Result<Vec<OffscreenId>, rusqlite::Error> {
        if !self.table_exists("Offscreen") {
            return Ok(vec![]);
        }

        let stmt = self
            .conn
            .prepare_cached("SELECT MainId FROM Offscreen WHERE LayerId=?1");

        stmt?.query_map([id.0], |r| r.get(0))?.collect()
    }

    pub fn get_vector_obj_list_ids_for_layer(
        &self,
        id: LayerId,
    ) -> Result<Vec<VectorObjListId>, rusqlite::Error> {
        if !self.table_exists("VectorObjectList") {
            return Ok(vec![]);
        }

        let stmt = self
            .conn
            .prepare_cached("SELECT MainId FROM VectorObjectList WHERE LayerId=?1");

        stmt?.query_map([id.0], |r| r.get(0))?.collect()
    }

    pub fn get_base_mipmap_offscreen(&self, id: MipmapId) -> Result<OffscreenId, rusqlite::Error> {
        let stmt = self.conn.prepare_cached(
            "SELECT Offscreen
            FROM MipmapInfo info
            INNER JOIN Mipmap mip
            ON info.MainId = mip.BaseMipmapInfo WHERE mip.MainId = ?1;",
        );

        stmt?.query_row([id.0], |r| r.get(0))
    }
}
