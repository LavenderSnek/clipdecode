use nom::bytes::complete::{tag, take};
use nom::combinator::verify;
use nom::IResult;
use nom::number::complete::be_u64;

#[derive(Debug, Copy, Clone)]
pub struct ClipHeader {
    // CFSCHUNK
    pub filesize: u64,
    // head_pos: u64
    //---
    // CHNKHead
    // data_size: u64 >= 40
    // ?: u64 = 256
    pub sqlite_chunk_pos: u64,
    // ?: u64 = 16
    // ?: &[u8; 16] ???
}

impl ClipHeader {
    pub fn parse(inp: &[u8]) -> IResult<&[u8], Self> {
        // CSFCHUNK
        let (i, _) = tag(b"CSFCHUNK")(inp)?;
        let (i, filesize) = be_u64(i)?;
        let (_, head_pos) = be_u64(i)?;

        // CHNKHead
        let (i, _) = take(head_pos)(inp)?;
        let (i, _) = tag("CHNKHead")(i)?;

        let (i, _) = verify(be_u64, |x| { *x >= 40 })(i)?;
        let (i, _) = verify(be_u64, |x| { *x == 256 })(i)?;

        let (i, sqlite_chunk_pos) = be_u64(i)?;

        let (i, _) = verify(be_u64, |x| { *x == 16 })(i)?;
        let (i, _) = take(16u32)(i)?;

        Ok((i, ClipHeader { filesize, sqlite_chunk_pos }))
    }
}
