pub(crate) mod bit_reader;
pub(crate) mod tone_curve;

pub(crate) fn u16(x: &[u8], is_le: bool) -> u16 {
    let bytes = [x[0], x[1]];
    if is_le {
        u16::from_le_bytes(bytes)
    } else {
        u16::from_be_bytes(bytes)
    }
}

