use nom::bytes::complete::{is_not, tag, take, take_while};
use nom::combinator::{all_consuming, cond, verify};
use nom::IResult;
use nom::multi::many1;
use nom::number::complete::be_u32;

mod tags {
    use std::sync::LazyLock;

    fn blockdata_tag(s: &'static str) -> Vec<u8> {
        let mut v: Vec<u8> = s.encode_utf16()
            .flat_map(|x| { x.to_be_bytes() })
            .collect();

        let mut sz = (s.len() as u32).to_be_bytes().to_vec();

        sz.append(&mut v);
        sz
    }

    pub(super) const BEGIN_CHUNK: LazyLock<Vec<u8>> = LazyLock::new(|| { blockdata_tag("BlockDataBeginChunk") });
    pub(super) const END_CHUNK: LazyLock<Vec<u8>> = LazyLock::new(|| { blockdata_tag("BlockDataEndChunk") });

    pub(crate) const STATUS: LazyLock<Vec<u8>> = LazyLock::new(|| { blockdata_tag("BlockStatus") });
    pub(crate) const CHECKSUM: LazyLock<Vec<u8>> = LazyLock::new(|| { blockdata_tag("BlockCheckSum") });
}

pub struct BlockData<'a> {
    // data size: u32
    pub prefix: [u8; 4], // ?: [x2 07 00 00] (possibly checksum)
    pub zlib_data: &'a [u8],
}

impl<'a> BlockData<'a> {
    pub fn parse(inp: &'a [u8]) -> IResult<&[u8], Self> {
        let (i, size) = be_u32(inp)?;
        let (remaining, data) = take(size)(i)?;

        let (zlib_data, prefix_slice) = take(4u32)(data)?;
        
        let (_,_) = tag([0x78])(zlib_data)?; // kind of verify zlib header

        //---
        let mut prefix = [0u8; 4];
        prefix.copy_from_slice(prefix_slice);

        Ok((remaining, BlockData { prefix, zlib_data }))
    }
}


pub struct BlockDataChunk<'a> {
    // size mark: u32 >= 104
    //----
    // sz + BlockDataBeginChunk utf16be
    // ?: [16 bytes]
    // u32: 0 || 1, data does not exist if this is 0, skip to end tag
    pub data: Option<BlockData<'a>>,
    // sz + BlockDataEndChunk utf16be
}

impl<'a> BlockDataChunk<'a> {
    
    fn parse_inner(inp: &'a [u8]) -> IResult<&[u8], Self> {
        let (i, _) = tag(tags::BEGIN_CHUNK.as_slice())(inp)?;

        let (i, _) = take(16u32)(i)?;

        let (i, data_flag) = verify(be_u32, |x| { *x == 0 || *x == 1 })(i)?;
        let (i, data) = cond(data_flag == 1, BlockData::parse)(i)?;

        let (i, _) = tag(tags::END_CHUNK.as_slice())(i)?;
        
        Ok((i, BlockDataChunk { data }))
    }
    
    pub fn parse(inp: &'a [u8]) -> IResult<&[u8], Self> {
        let (_, size) = verify(be_u32, |x| { *x >= 104 })(inp)?;
      
        let (remaining, inner) = take(size)(inp)?;
        let (inner, _) = be_u32(inner)?; // remove the size byte from inner chunk
   
        let (_, dc) = all_consuming(Self::parse_inner)(inner)?;
        
        Ok((remaining, dc))
    }
}

pub struct BlockDataSection<'a> {
    pub chunks: Vec<BlockDataChunk<'a>>, // [BlockDataChunk] many1(Blockdata)?
    // sz + BlockStatus utf16be
    // ?: u32 = 12
    // c1: u32 // one of these might be a chunk count
    // c2: u32
    // [u8; c1 * c2]
    // sz + BlockCheckSum utf16be
    // ?: u32 = 12
    // c1: u32
    // c2: u32
    // [u8; c1 * c2]
    // pub checksum_data: [u8; 28|16],
}

impl<'a> BlockDataSection<'a> {
    fn parse_status_checksum_body(inp: &[u8]) -> IResult<&[u8], &[u8]> {
        
        let (i, _) = verify(be_u32, |x| { *x == 12 })(inp)?;
        let (i, s1) = be_u32(i)?;
        let (i, s2) = be_u32(i)?;
        
        take(s1 * s2)(i)
    }
 
    pub fn parse(inp: &'a [u8]) -> IResult<&[u8], Self> {
        let (i, chunks) = many1(BlockDataChunk::parse)(inp)?;
        
        // todo: actually use the checksums
        
        let (i, _) = tag(tags::STATUS.as_slice())(i)?;
        let (i, _status) = Self::parse_status_checksum_body(i)?;

        let (i, _) = tag(tags::CHECKSUM.as_slice())(i)?;
        let (i, _checksum) = Self::parse_status_checksum_body(i)?;
        
        Ok((i, BlockDataSection { chunks }))
    }
}

