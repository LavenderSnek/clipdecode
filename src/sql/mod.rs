use rusqlite::Connection;

mod canvas;
mod dbutil;
mod layer;

// db wrapper for csp
pub struct ClipDb<'a> {
    conn: &'a Connection,
}

impl<'a> ClipDb<'a> {
    pub fn with_conn(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> ClipDb<'a> {
    pub fn conn(&self) -> &Connection {
        self.conn
    }

    /// get external chunk offset for the given external id
    pub fn get_exta_chunk_offset(&self, ext_id: &str) -> Option<i64> {
        let stmt = self
            .conn
            .prepare_cached("SELECT Offset FROM ExternalChunk WHERE ExternalID=?1");
        stmt.unwrap()
            .query_row([ext_id], |r| {
                let v: i64 = r.get(0).unwrap();
                Ok(v)
            })
            .ok()
    }

    /// checks whether a table exists
    pub fn table_exists(&self, name: &str) -> bool {
        let stmt = self
            .conn
            .prepare_cached("SELECT name FROM sqlite_master WHERE type='table' AND name=?1");
        stmt.unwrap().exists([name]).unwrap()
    }
}
