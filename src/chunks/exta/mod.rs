use std::io::Read;

use nom::bytes::complete::{tag, take};
use nom::combinator::verify;
use nom::IResult;
use nom::number::complete::be_u64;
use crate::ExtaOffscreen;

pub mod offscreen;
mod vector;

pub enum ClipExtaBody<'a> {
    Offscreen(ExtaOffscreen<'a>), // block data
    // VectorObjects,
    // ModelBank3D,
    // ModelLoader3D,
    // Track, // is there 2 types of this? (or is it just 2 of the same type)
    // ItemBinary,
    // SceneData3D,
    // ModelData3D,
    // TimeLapse
    Unknown,
}

impl<'a> ClipExtaBody<'a> {}

#[derive(Debug, Copy, Clone)]
pub struct ClipExtaHeader {
    // CHNKExta
    pub chunk_size: u64,
    // ext_len: u64 = 40
    pub ext_id: [u8; 40],
    pub body_size: u64,
}

impl ClipExtaHeader {
    pub fn parse(inp: &[u8]) -> IResult<&[u8], Self> {
        // CHNKExta
        let (i, _) = tag("CHNKExta")(inp)?;

        let (i, chunk_size) = be_u64(i)?;

        let (i, _) = verify(be_u64, |x| { *x == 40 })(i)?;
        let (i, ext_id_slice) = take(40u64)(i)?;

        let (i, body_size) = verify(be_u64, |x| { *x < i32::MAX as _ && *x == chunk_size - 56 })(i)?;

        //---
        let mut ext_id = [0u8; 40];
        ext_id.copy_from_slice(ext_id_slice);

        Ok((i, ClipExtaHeader { chunk_size, ext_id, body_size }))
    }
}
