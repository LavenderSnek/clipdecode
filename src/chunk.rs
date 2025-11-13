use binrw::{binread, binwrite};

#[binread]
#[binwrite]
#[brw(big)]
#[brw(magic = b"CSFCHUNK")]
#[derive(Debug, Clone, Copy)]
pub struct CsfChunk {
    pub filesize: u64,
    pub head_chunk_pos: u64,
}

#[binread]
#[binwrite]
#[brw(big)]
#[brw(magic = b"CHNKHead")]
#[derive(Debug, Clone, Copy)]
pub struct HeadChunk {
    #[br(temp, assert( _size == 40 ))]
    #[bw(calc = 40)]
    _size: u64,

    #[br(temp, assert( _v256 == 256 ))]
    #[bw(calc = 256)]
    _v256: u64,

    pub sqlite_chunk_pos: u64,

    #[br(temp, assert( _guid_size == 16 ))]
    #[bw(calc = 16)]
    _guid_size: u64,

    pub guid: [u8; 16], // changing this doesn't do anything
}

#[binread]
#[binwrite]
#[brw(big)]
#[brw(magic = b"CHNKExta")]
#[derive(Debug, Clone, Copy)]
pub struct ExtaChunkHeader {
    #[br(temp)]
    #[bw(calc = body_size + 56)]
    _size: u64,

    #[br(temp, assert(_ext_id_len == 40))]
    #[bw(calc = 40)]
    _ext_id_len: u64,

    pub ext_id: [u8; 40],
    pub body_size: u64,
}

#[binread]
#[binwrite]
#[brw(big)]
#[brw(magic = b"CHNKSQLi")]
#[derive(Debug, Clone, Copy)]
pub struct SqliteChunkHeader {
    pub body_size: u64,
}

#[binread]
#[binwrite]
#[brw(big)]
#[brw(magic = b"CHNKFoot")]
#[derive(Default, Debug, Clone, Copy)]
pub struct FootChunk {
    #[br(temp, assert( _end == 0 ))]
    #[bw(calc = 0)]
    _end: u64,
}
