use super::{Decode, Preprocess};
use crate::parse::{dng::DngInfo, DecodingInfo};
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
        }
    }
    fn decode_with_preprocess<RS: Read + Seek>(&self, reader: RS) -> Result<Box<[u16]>, erreport::Report> {
        todo!()
    }
}
