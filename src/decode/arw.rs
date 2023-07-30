use super::{general_16bit_iter, Decode, Preprocess};
use crate::{
    parse::{arw::ArwInfo, DecodingInfo},
    report::{Report, ToReport},
    Error,
};

impl Preprocess for ArwInfo {
    fn black_level_substract(&self, x: u16) -> u16 {
        x.saturating_sub(self.black_level)
    }
    fn white_level_scaleup(&self, x: u16) -> u16 {
        x * 4
    }
}

impl Decode<ArwInfo> for ArwInfo {
    fn to_decoding_info(self) -> DecodingInfo {
        DecodingInfo {
            width: self.width,
            height: self.height,
            white_balance: self.white_balance,
            cfa_pattern: self.cfa_pattern,
        }
    }
    fn decode_with_preprocess(&self, strip_bytes: Box<[u8]>) -> Result<Box<[u16]>, Report> {
        match self.compression {
            1 => {
                let image = general_16bit_iter(&strip_bytes, self.is_le)
                    .map(|v| self.bl_then_wl(v))
                    .collect();
                Ok(image)
            }
            c => Err(Error::UnknownCompression(c)).to_report(),
        }
    }
}
