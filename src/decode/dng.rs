use super::{general_16bit_iter, Decode, Preprocess};
use crate::{
    parse::{
        dng::{DngError, DngInfo},
        get_bytes, DecodingInfo,
    },
    ToReport,
};
use erreport::Report;
use std::io::{Read, Seek};

impl Preprocess for DngInfo {
    fn black_level_substract(&self, x: u16) -> u16 {
        x.saturating_sub(self.black_level)
    }
    fn white_level_scaleup(&self, x: u16) -> u16 {
        x << self.scaleup_factor
    }
}

impl Decode for DngInfo {
    fn to_decoding_info(self) -> DecodingInfo {
        DecodingInfo {
            width: self.width,
            height: self.height,
            white_balance: self.white_balance,
            cfa_pattern: self.cfa_pattern,
            color_matrix: Some(self.color_matrix_2),
        }
    }
    fn decode_with_preprocess<RS: Read + Seek>(
        &self,
        mut reader: RS,
    ) -> Result<Box<[u16]>, Report> {
        match (self.compression, self.cfa_pattern.as_ref()) {
            (1, _) => {
                // uncompressed bayer or uncompressed rgb
                let strip_bytes =
                    get_bytes(&mut reader, self.strip_addr, self.strip_size).to_report()?;
                let image = general_16bit_iter(&strip_bytes, self.is_le)
                    .map(|v| self.bl_then_wl(v))
                    .collect();
                Ok(image)
            }
            (7, None) => {
                // lossless compressed rgb
                todo!()
            }
            (7, _) => {
                // lossless compressed bayer
                todo!()
            }
            _ => Err(DngError::CompressionTypeNotSupported(self.compression)).to_report(),
        }
    }
}
