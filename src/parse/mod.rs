use crate::report::{Report, ToReport};
use std::io::{Read, Seek, SeekFrom};

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
