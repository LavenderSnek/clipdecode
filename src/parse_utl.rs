use binrw::{binread, binwrite};

fn encode_utf16_be(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|x| x.to_be_bytes()).collect()
}

#[binread]
#[binwrite]
#[brw(big)]
#[brw(import(txt: &str))]
#[derive(Debug, Clone, Copy)]
pub struct Utf16BeTag {
    #[br(temp, assert(txt.len() == _chars as _))]
    #[bw(calc = txt.len() as _)]
    _chars: u32,

    #[br(temp, count = _chars * 2)]
    #[br(assert(_body == encode_utf16_be(txt)))]
    #[bw(calc = encode_utf16_be(txt))]
    _body: Vec<u8>,

    #[br(calc = 4 + (_chars * 2))]
    #[bw(ignore)]
    size: u32,
}

impl Utf16BeTag {
    pub fn calc_size(&self) -> u32 {
        self.size
    }
}

#[cfg(test)]
mod test {
    use crate::parse_utl::Utf16BeTag;
    use binrw::{binread, binwrite, BinRead, BinWrite};
    use std::io::Cursor;

    #[binread]
    #[binwrite]
    #[brw(little)]
    struct HiMagic {
        #[brw(args("Hi"))]
        tag: Utf16BeTag,
        data: u8,
    }

    #[test]
    fn utf16be_tag_normal() {
        let data = b"\0\0\0\x02\0H\0i\x03";

        let mut rc = Cursor::new(data);
        let r = HiMagic::read(&mut rc).unwrap();

        assert_eq!(r.tag.calc_size(), 8);
        assert_eq!(r.data, 3);

        let mut wc = Cursor::new(vec![]);
        let _ = r.write(&mut wc).unwrap();
        assert_eq!(wc.into_inner(), data);
    }

    #[test]
    fn utf16be_tag_read_wrong_prefix() {
        let data = b"\0\0\0\x03\0H\0i\x03";

        let mut rc = Cursor::new(data);
        let r = HiMagic::read(&mut rc);

        assert!(r.is_err())
    }

    #[test]
    fn utf16be_tag_read_wrong_body() {
        let data = b"\0\0\0\x02\0Q\0i\x03";

        let mut rc = Cursor::new(data);
        let r = HiMagic::read(&mut rc);

        assert!(r.is_err())
    }
}
