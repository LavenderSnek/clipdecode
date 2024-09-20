use num_enum::TryFromPrimitive;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use crate::ClipDb;

#[derive(Debug, Copy, Clone)]
pub struct Canvas {
    pub id: i64,
    pub unit: CanvasUnit,
    pub width: f64,
    pub height: f64,
    pub resolution_dpi: f64,
    // channel_bytes: i64,
    // default_channel_order: i64,
    // root_folder_id: i64,
    pub current_layer_id: i64,
    // there's more but idk what they mean yet
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, TryFromPrimitive)]
#[repr(i64)]
pub enum CanvasUnit {
    Pixels = 0,
    Centimetres = 1,
    Millimetres = 2,
    Inches = 3,
    Points = 5,
    #[num_enum(catch_all)]
    Unknown(i64),
} // why is there no 4??

impl FromSql for CanvasUnit {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        CanvasUnit::try_from(value.as_i64()?).map_err(|_| { FromSqlError::InvalidType })
    }
}

impl<'a> ClipDb<'a> {
    /// The raw image preview data for the given canvas
    pub fn get_preview_image_for_canvas(&self, canvas_id: i64) -> Option<Vec<u8>> {
        let stmt = self.conn().prepare_cached("SELECT ImageData from CanvasPreview where CanvasId=?1");

        stmt.unwrap().query_row([canvas_id], |r| {
            let v = r.get(0).unwrap();
            Ok(v)
        }).ok()
    }

    /// returns a list of all available canvas ids
    pub fn get_canvas_ids(&self) -> Vec<i64> {
        let stmt = self.conn().prepare_cached("SELECT MainId from Canvas");

        stmt.unwrap().query_map([], |r| {
            Ok(r.get(0)?)
        }).unwrap().map(|r| { r.unwrap() }).collect()
    }

    /// get the canvas for the given canvas ID
    pub fn get_canvas(&self, canvas_id: i64) -> Option<Canvas> {
        let stmt = self.conn().prepare_cached("SELECT \
                MainId, \
                CanvasUnit, \
                CanvasWidth, \
                CanvasHeight, \
                CanvasResolution, \
                CanvasCurrentLayer \
            FROM Canvas WHERE MainId=?1");

        stmt.unwrap().query_row([canvas_id], |r| {
            Ok(Canvas {
                id: r.get(0).unwrap(),
                unit: r.get(1).unwrap(),
                width: r.get(2).unwrap(),
                height: r.get(3).unwrap(),
                resolution_dpi: r.get(4).unwrap(),
                current_layer_id: r.get(5).unwrap(),
            })
        }).ok()
    }
}