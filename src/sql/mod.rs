use rusqlite::{
    types::{FromSql, FromSqlError},
    Connection,
};

pub mod canvas;
pub(crate) mod dbutil;
pub mod external;
pub mod layer;

macro_rules! def_sql_id {
    ($tp_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Eq, PartialEq, Copy, Clone)]
        pub struct $tp_name(pub i64);

        impl rusqlite::types::FromSql for $tp_name {
            fn column_result(
                value: rusqlite::types::ValueRef<'_>,
            ) -> rusqlite::types::FromSqlResult<Self> {
                Ok($tp_name(value.as_i64()?))
            }
        }
    };
}

def_sql_id!(CanvasId);
def_sql_id!(LayerId);
def_sql_id!(OffscreenId);
def_sql_id!(VectorObjListId);

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ExtaChunkId(pub String);

impl FromSql for ExtaChunkId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s = value.as_blob()?.to_vec();
        Ok(ExtaChunkId(
            String::from_utf8(s).map_err(|e| FromSqlError::Other(Box::new(e)))?,
        ))
    }
}

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

    /// checks whether a table exists
    pub fn table_exists(&self, name: &str) -> bool {
        let stmt = self
            .conn
            .prepare_cached("SELECT name FROM sqlite_master WHERE type='table' AND name=?1");
        stmt.unwrap().exists([name]).unwrap()
    }
}
