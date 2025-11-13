use binrw::{binread, binwrite, BinRead};

fn make_blockdata_tag_bytes(s: &str) -> Vec<u8> {
    let mut utf16_bytes: Vec<u8> = s.encode_utf16().flat_map(|x| x.to_be_bytes()).collect();
    let mut bytes = (s.len() as u32).to_be_bytes().to_vec();
    bytes.append(&mut utf16_bytes);

    return bytes;
}

#[binrw::parser(reader)]
fn parse_blockdata_tag(s: &str) -> binrw::BinResult<Vec<u8>> {
    let expected = make_blockdata_tag_bytes(s);

    let rpos = reader.stream_position()?;

    let mut buf = vec![0u8; expected.len()];
    reader.read_exact(&mut buf)?;

    if buf == expected {
        Ok(buf)
    } else {
        Err(binrw::Error::BadMagic {
            pos: rpos,
            found: Box::new(buf),
        })
    }
}

#[binread]
#[binwrite]
#[brw(big)]
pub struct BlockDataChunk {
    #[br(temp)]
    #[bw(calc = self.calc_size())]
    _size: u32, // size of the chunk, including the size itself

    #[br(temp, args("BlockDataBeginChunk"))]
    #[br(parse_with = parse_blockdata_tag)]
    #[bw(calc = make_blockdata_tag_bytes("BlockDataBeginChunk"))]
    _begin_chunk: Vec<u8>,

    pub unknown: [u8; 16],

    #[br(temp)]
    #[bw(calc = if data.is_some() { 1 } else { 0 })]
    data_flag: u32,

    #[br(if(data_flag == 1))]
    pub data: Option<BlockData>,

    #[br(temp, args("BlockDataEndChunk"))]
    #[br(parse_with = parse_blockdata_tag)]
    #[bw(calc = make_blockdata_tag_bytes("BlockDataEndChunk"))]
    _end_chunk: Vec<u8>,
}

impl BlockDataChunk {
    fn calc_size(&self) -> u32 {
        4 // size
            + make_blockdata_tag_bytes("BlockDataBeginChunk").len() as u32
            + self.unknown.len() as u32
            + 4 // data flag
            + self.data.as_ref().map_or(0, |d| d.calc_size()) as u32
            + make_blockdata_tag_bytes("BlockDataEndChunk").len() as u32
    }
}

#[binread]
#[binwrite]
#[brw(big)]
pub struct BlockData {
    #[br(temp)]
    #[bw(calc = self.calc_size() - 4)]
    _size: u32, // size excluding size

    // in LE for some reason
    #[brw(little)]
    #[br(temp, assert(data_size + 4 == _size))]
    #[bw(calc = compressed_data.len() as u32)]
    data_size: u32,

    #[br(count = data_size)]
    pub compressed_data: Vec<u8>,
}

impl BlockData {
    fn calc_size(&self) -> u32 {
        4 + self.compressed_data.len() as u32 + 4
    }
}

#[binread]
#[binwrite]
#[brw(big)]
#[derive(Debug)]
pub struct BlockStatus {
    #[br(temp, args("BlockStatus"))]
    #[br(parse_with = parse_blockdata_tag)]
    #[bw(calc = make_blockdata_tag_bytes("BlockStatus"))]
    _tag: Vec<u8>,

    #[br(temp, assert(_v12 == 12))]
    #[bw(calc = 12)]
    _v12: u32,

    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    block_count: u32,

    #[br(temp, assert(_width == 4))]
    #[bw(calc = 4)]
    _width: u32,

    #[br(count = block_count)]
    pub entries: Vec<[u8; 4]>,
}

#[binread]
#[binwrite]
#[brw(big)]
#[derive(Debug)]
pub struct BlockChecksum {
    #[br(temp, args("BlockCheckSum"))]
    #[br(parse_with = parse_blockdata_tag)]
    #[bw(calc = make_blockdata_tag_bytes("BlockCheckSum"))]
    _tag: Vec<u8>,

    #[br(temp, assert(_v12 == 12))]
    #[bw(calc = 12)]
    _v12: u32,

    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    block_count: u32,

    #[br(temp, assert(_width == 4))]
    #[bw(calc = 4)]
    _width: u32,

    #[br(count = block_count)]
    pub entries: Vec<[u8; 4]>,
}

#[binrw::parser(reader, endian)]
fn parse_blockdata_list() -> binrw::BinResult<Vec<BlockDataChunk>> {
    let mut v = vec![];

    v.push(BlockDataChunk::read_options(reader, endian, ())?);

    loop {
        let block = BlockDataChunk::read_options(reader, endian, ());

        if let Ok(b) = block {
            v.push(b);
        } else {
            return Ok(v);
        }
    }
}

#[binread]
#[binwrite]
#[brw(big)]
pub struct ExtaOffscreen {
    #[br(parse_with = parse_blockdata_list)]
    pub blocks: Vec<BlockDataChunk>,
    pub status: BlockStatus,
    pub cheksum: BlockChecksum,
}
