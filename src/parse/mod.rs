use crate::report::{Report, ToReport};
use std::io::{Read, Seek, SeekFrom};

pub(crate) mod arw;

pub(crate) fn get_bytes<T: Read + Seek>(
    mut reader: T,
    addr: u64,
    size: usize,
) -> Result<Vec<u8>, Report> {
    let mut bytes = Vec::with_capacity(size);
    reader.seek(SeekFrom::Start(addr)).to_report()?;
    reader.read_exact(&mut bytes).to_report()?;

    Ok(bytes)
}
