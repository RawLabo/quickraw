use crate::report::{Report, ToReport};
use std::io::{Read, Seek, SeekFrom};

pub struct WhiteBalance {
    pub(crate) r: u32,
    pub(crate) g: u32,
    pub(crate) b: u32,
    pub(crate) bit_shift: u32,
}
impl From<[u16; 3]> for WhiteBalance {
    fn from([r, g, b]: [u16; 3]) -> Self {
        let mut bit_shift = 0u32;
        for i in 1u32.. {
            if (g >> i) == 1 {
                bit_shift = i;
                break;
            }
        }

        Self {
            r: r as u32,
            g: g as u32,
            b: b as u32,
            bit_shift,
        }
    }
}

pub enum CFAPattern {
    RGGB,
    GRBG,
    GBRG,
    BGGR,
    XTrans0, // RBGBRG
    XTrans1, // GGRGGB
}
impl<'a> From<&'a [u8]> for CFAPattern {
    fn from(value: &'a [u8]) -> Self {
        match value {
            [0, 1, 1, 2] => CFAPattern::RGGB,
            [2, 1, 1, 0] => CFAPattern::BGGR,
            [1, 0, 2, 1] => CFAPattern::GRBG,
            [1, 2, 0, 1] => CFAPattern::GBRG,
            _ => CFAPattern::RGGB,
        }
    }
}

pub(crate) mod arw;

pub(crate) fn get_bytes<T: Read + Seek>(
    mut reader: T,
    addr: u64,
    size: usize,
) -> Result<Vec<u8>, Report> {
    let mut bytes = vec![0u8; size];
    reader.seek(SeekFrom::Start(addr)).to_report()?;
    reader.read_exact(&mut bytes).to_report()?;

    Ok(bytes)
}
