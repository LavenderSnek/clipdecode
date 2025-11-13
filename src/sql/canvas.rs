use crate::sql::ClipDb;
use num_enum::TryFromPrimitive;
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ValueRef},
    Row,
};

#[derive(Debug, Copy, Clone)]
pub struct Canvas {
    pub id: i64,
    pub unit: CanvasUnit,
    pub width: f64,
    pub height: f64,
    pub resolution: f64,
    pub channel_bytes: i64,
    // default_channel_order: i64,
    pub root_folder_id: i64,
    pub current_layer_id: i64,
    // there's more but idk what they mean yet
}

impl Canvas {
    fn from_row(r: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Canvas {
            id: r.get("MainId")?,
            unit: r.get("CanvasUnit")?,
            width: r.get("CanvasWidth")?,
            height: r.get("CanvasHeight")?,
            resolution: r.get("CanvasResolution")?,
            channel_bytes: r.get("CanvasChannelBytes")?,
            root_folder_id: r.get("CanvasRootFolder")?,
            current_layer_id: r.get("CanvasCurrentLayer")?,
        })
    }
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
} // there's no 4

impl FromSql for CanvasUnit {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        CanvasUnit::try_from(value.as_i64()?).map_err(|_| FromSqlError::InvalidType)
    }
}

pub struct CanvasPreview {
    pub id: i64,
    pub canvas_id: i64,
    pub image_type: i64,
    pub image_width: i64,
    pub image_height: i64,
    pub image_data: Vec<u8>,
}

impl CanvasPreview {
    fn from_row(r: &Row) -> Result<Self, rusqlite::Error> {
        Ok(CanvasPreview {
            id: r.get("MainId")?,
            canvas_id: r.get("CanvasId")?,
            image_type: r.get("ImageType")?,
            image_width: r.get("ImageWidth")?,
            image_height: r.get("ImageHeight")?,
            image_data: r.get("ImageData")?,
        })
    }
}

impl<'a> ClipDb<'a> {
    /// The raw image preview data for the given canvas
    pub fn get_preview_image_for_canvas(
        &self,
        canvas_id: i64,
    ) -> Result<CanvasPreview, rusqlite::Error> {
        let stmt = self
            .conn()
            .prepare_cached("SELECT * from CanvasPreview where CanvasId=?1");

        stmt?.query_row([canvas_id], CanvasPreview::from_row)
    }

    /// returns a list of all available canvas ids
    pub fn get_canvas_ids(&self) -> Result<Vec<i64>, rusqlite::Error> {
        let stmt = self.conn().prepare_cached("SELECT MainId from Canvas");
        stmt?.query_map([], |r| r.get(0))?.collect()
    }

    /// get the canvas for the given canvas ID
    pub fn get_canvas(&self, canvas_id: i64) -> Result<Canvas, rusqlite::Error> {
        let stmt = self
            .conn()
            .prepare_cached("SELECT * FROM Canvas WHERE MainId=?1");

        stmt?.query_row([canvas_id], Canvas::from_row)
    }
}
