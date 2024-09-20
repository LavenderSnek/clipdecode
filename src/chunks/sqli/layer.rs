use nom::bytes::complete::take;
use nom::IResult;
use nom::number::complete::{be_i32, be_u32};
use num_enum::TryFromPrimitive;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};

use crate::ClipDb;

// a lot of these actually have smaller possible values
// we're just going with the largest that would fit for consistency

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum FilterLayerInfo {
    // FilterLayerInfo
    // kind: u32
    // size: u32
    // data
    BrightnessContrast(i32, i32), // 1
    LevelCorrection, // 2 todo
    ToneCurve, // 3 todo
    Hsl(i32, i32, i32), // 4
    ColorBalance, // 5 todo
    ReverseGradient, // 6
    Posterization(u32), // 7
    Binarization(u32), // 8
    GradientMap, // 9 todo
    Unknown(u32),
}

impl FilterLayerInfo {
    fn parse_brightness_contrast(data: &[u8]) -> IResult<&[u8], Self> {
        let (i, brightness) = be_i32(data)?;
        let (i, contrast) = be_i32(i)?;
        Ok((i, FilterLayerInfo::BrightnessContrast(brightness, contrast)))
    }

    fn parse_level_correction(data: &[u8]) -> IResult<&[u8], Self> {
        Ok((data, FilterLayerInfo::LevelCorrection))
    }

    fn parse_tone_curve(data: &[u8]) -> IResult<&[u8], Self> {
        Ok((data, FilterLayerInfo::ToneCurve))
    }

    fn parse_hsl(data: &[u8]) -> IResult<&[u8], Self> {
        let (i, h) = be_i32(data)?;
        let (i, s) = be_i32(i)?;
        let (i, l) = be_i32(i)?;
        Ok((i, FilterLayerInfo::Hsl(h, s, l)))
    }

    fn parse_color_balance(data: &[u8]) -> IResult<&[u8], Self> {
        Ok((data, FilterLayerInfo::ColorBalance))
    }

    fn parse_gradient_map(data: &[u8]) -> IResult<&[u8], Self> {
        Ok((data, FilterLayerInfo::GradientMap))
    }

    /// parse from filter layer info
    pub fn parse(inp: &[u8]) -> IResult<&[u8], Self> {
        let (i, kind) = be_u32(inp)?;
        let (i, size) = be_u32(i)?;
        let (i, data) = take(size)(i)?;

        match kind {
            1 => Self::parse_brightness_contrast(data),
            2 => Self::parse_level_correction(data),
            3 => Self::parse_tone_curve(data),
            4 => Self::parse_hsl(data),
            5 => Self::parse_color_balance(data),
            6 => Ok((i, FilterLayerInfo::ReverseGradient)),
            7 => Ok((i, FilterLayerInfo::Posterization(be_u32(data)?.1))),
            8 => Ok((i, FilterLayerInfo::Binarization(be_u32(data)?.1))),
            9 => Self::parse_gradient_map(data),
            _ => Ok((i, FilterLayerInfo::Unknown(kind)))
        }
    }
}

impl FromSql for FilterLayerInfo {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let (_, v) = Self::parse(value.as_bytes()?).unwrap();
        Ok(v)
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
    VividLight = 17,
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
        LayerBlendMode::try_from(value.as_i64()?).map_err(|_| { FromSqlError::InvalidType })
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, TryFromPrimitive)]
#[repr(i64)]
pub enum LayerKind {
    Dummy = 256, // root folder
    Paper = 1584, // paper layer, only one per canvas
    Other = 0, // vector, folder, 3d, folder, frame folder
    Raster = 1,
    Fill = 2, // gradient, fill, tone
    Filter = 4098,
    #[num_enum(catch_all)]
    Unknown(i64),
}

impl FromSql for LayerKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        LayerKind::try_from(value.as_i64()?).map_err(|_| { FromSqlError::InvalidType })
    }
}

// todo: needs more decoding
pub struct Layer {
    pub id: i64,
    pub canvas_id: i64,
    pub name: String,
    pub kind: LayerKind,
    pub alpha: i64, // this actually only goes to 256
    pub blend_mode: LayerBlendMode,
}

impl<'a> ClipDb<'a> {
    // i give up- im bored with sql
    fn get_ext_id_offsets_for_layer(&self, table_name: &str, ext_id_colum_name: &str, layer_id: i64) -> Vec<i64> {
        if !self.table_exists(table_name) {
            return vec![];
        }

        let stmt = self.conn.prepare_cached(&format!(
            "select ExternalChunk.Offset from ExternalChunk \
            inner join {table_name} on hex(ExternalChunk.ExternalID) = hex({table_name}.{ext_id_colum_name}) \
            where {table_name}.LayerId = ?1"
        ));

        stmt.unwrap().query_map([layer_id], |r| {
            Ok(r.get(0)?)
        }).unwrap().map(|r| { r.unwrap() }).collect()
    }

    pub fn get_offscreen_exta_offsets(&self, layer_id: i64) -> Vec<i64> {
        self.get_ext_id_offsets_for_layer("Offscreen", "BlockData", layer_id)
    }

    /// gets layers in the canvas with the given canvas ID
    pub fn get_layer_ids_for_canvas(&self, canvas_id: i64) -> Vec<i64> {
        let stmt = self.conn.prepare_cached("SELECT MainId FROM Layer WHERE CanvasId=?1");

        stmt.unwrap().query_map([canvas_id], |r| {
            Ok(r.get(0)?)
        }).unwrap().map(|r| { r.unwrap() }).collect()
    }

    /// gets the layer for the given ID if any
    pub fn get_layer(&self, layer_id: i64) -> Option<Layer> {
        let stmt = self.conn().prepare_cached("SELECT \
                MainId, \
                CanvasId, \
                LayerName, \
                LayerType, \
                LayerOpacity,\
                LayerComposite \
            FROM Layer WHERE MainId=?1");

        stmt.unwrap().query_row([layer_id], |r| {
            Ok(Layer {
                id: r.get(0).unwrap(),
                canvas_id: r.get(1).unwrap(),
                name: r.get(2).unwrap(),
                kind: r.get(3).unwrap(),
                alpha: r.get(4).unwrap(),
                blend_mode: r.get(5).unwrap(),
            })
        }).ok()
    }

    pub fn get_fiter_layer_info(&self, layer_id: i64) -> Option<FilterLayerInfo> {
        let stmt = self.conn().prepare_cached("SELECT FilterLayerInfo FROM Layer WHERE MainId=?1");
        stmt.unwrap().query_row([layer_id], |r| {
            let f: FilterLayerInfo = r.get(0)?;
            Ok(f)
        }).ok()
    }
}
