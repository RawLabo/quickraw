use super::{general_16bit_iter, Preprocess};
use crate::{
    parse::arw::ArwInfo,
    report::{Report, ToReport},
    Error,
};

impl super::Preprocess for ArwInfo {
    fn black_level_substract(&self, x: u16) -> u16 {
        x.saturating_sub(self.black_level)
    }
    fn white_level_scaleup(&self, x: u16) -> u16 {
        x * 4
    }
}


pub(crate) fn decode_with_preprocess(
    info: &ArwInfo,
    strip_bytes: Box<[u8]>,
) -> Result<Box<[u16]>, Report> {
    match info.compression {
        1 => {
            let image = general_16bit_iter(&strip_bytes, info.is_le)
                .map(|v| info.bl_then_wl(v))
                .collect();
            Ok(image)
        }
        c => Err(Error::UnknownCompression(c)).to_report(),
    }
}
