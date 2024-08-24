use nom::bytes::complete::{tag, take};
use nom::IResult;
use nom::number::complete::be_u64;
use rusqlite::Connection;
use rusqlite::types::FromSql;

mod canvas;
mod layer;

pub struct ClipSqliteChunk {
    size: u64
}

impl ClipSqliteChunk {
    /// consumes and parses just the header section of a sqlite chunk
    pub fn parse_header(inp: & [u8]) -> IResult<&[u8], Self> {
        let (i, _) = tag(b"CHNKSQLi")(inp)?;
        let (i, size) = be_u64(i)?;
        
        let (_, _) = tag(b"SQLite format 3\0")(i)?; // verify sqlite header 

        Ok((i, ClipSqliteChunk { size })) // maybe a bit redundant
    }

    /// consumes an entire sqlite chunk and returns just the data
    pub fn extract_data(inp: & [u8]) -> IResult<&[u8], &[u8]> {
        let (i, h) = ClipSqliteChunk::parse_header(inp)?;
        take(h.size)(i)
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
 
    /// get external chunk offset for the given external id bytes
    pub fn get_exta_chunk_offset(&self, ext_id:  &[u8; 40]) -> Option<i64> {
        let stmt = self.conn.prepare_cached("SELECT Offset FROM ExternalChunk WHERE ExternalID=?1");
        let ext = std::str::from_utf8(ext_id).unwrap();
        
        stmt.unwrap().query_row([ext], |r| {
            let v: i64 = r.get(0).unwrap();
            Ok(v)
        }).ok()
    }
    
    /// checks whether a table exists
    pub fn table_exists(&self, name: &str) -> bool {
        let stmt = self.conn.prepare_cached("SELECT name FROM sqlite_master WHERE type='table' AND name=?1");
        stmt.unwrap().exists([name]).unwrap()
    }
}

