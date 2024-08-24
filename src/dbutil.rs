use std::marker::PhantomData;
use rusqlite::{Connection, ffi};
use rusqlite::Error::SqliteFailure;

pub struct BorrowedConnection<'a> {
    pub conn: Connection,
    data: PhantomData<&'a [u8]>,
}

impl<'a> BorrowedConnection<'a> {
    // allows us to not copy the bytes
    pub fn from(data: &'a [u8]) -> Result<Self, rusqlite::Error> {
        #[allow(unused_mut)]
        let mut conn = Connection::open_in_memory()?;

        // SAFETY: Sqlite allows us to be responsible for allocating/freeing the memory 
        // when not using FREEONCLOSE
        // the struct (and therefore the connection) cant outlive the data (lifetime 'a)
        let rc = unsafe {
            let dp = data.as_ptr() as *mut u8;
            let flags = ffi::SQLITE_DESERIALIZE_READONLY; // dont free it (we dont own the data)
            ffi::sqlite3_deserialize(conn.handle(), c"main".as_ptr(), dp, data.len() as _, data.len() as _, flags)
        };

        if rc != ffi::SQLITE_OK {
            // rusqlite does way more error handling- but this is fine for now
            return Err(SqliteFailure(ffi::Error::new(rc), Some(String::from("Failed to open db"))));
        }

        Ok(Self { conn, data: PhantomData })
    }
}
