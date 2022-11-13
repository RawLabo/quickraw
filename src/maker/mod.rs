use crate::decode::{CFAPattern, Crop, Orientation};
use thiserror::Error;

pub(super) mod selector;
mod utility;

mod adobe;
mod decode_utility;
mod fujifilm;
mod nikon;
mod olympus;
mod panasonic;
mod sony;

pub(super) trait RawDecoder {
    fn new(info: quickexif::ParsedInfo) -> Self
    where
        Self: Sized;
    fn get_info(&self) -> &quickexif::ParsedInfo;
    fn into_info(self) -> quickexif::ParsedInfo;
    fn get_white_balance(&self) -> Result<[i32; 3], DecodingError> {
        let info = self.get_info();
        Ok([
            info.i32("white_balance_r")?,
            info.i32("white_balance_g")?,
            info.i32("white_balance_b")?,
        ])
    }
    fn get_crop(&self) -> Option<Crop>;
    fn get_bps_scale(&self) -> Result<u16, DecodingError> {
        let bps = self.get_info().u16("bps")?;
        let result = match bps {
            12 => 16,
            14 => 4,
            _ => 1,
        };
        Ok(result)
    }
    fn get_orientation(&self) -> Orientation {
        match self.get_info().u16("orientation").ok() {
            None => Orientation::Horizontal,
            Some(o) => match o {
                1 => Orientation::Horizontal,
                3 => Orientation::Rotate180,
                6 => Orientation::Rotate90,
                8 => Orientation::Rotate270,
                _ => Orientation::Horizontal,
            },
        }
    }
    fn decode_with_preprocess(&self, buffer: &[u8]) -> Result<Vec<u16>, DecodingError>;
    fn get_thumbnail<'a>(&self, buffer: &'a [u8]) -> Result<&'a [u8], DecodingError>;
    fn get_cfa_pattern(&self) -> Result<CFAPattern, DecodingError> {
        let cfa_pattern = self.get_info().u8a4("cfa_pattern")?;
        let result = match cfa_pattern {
            [0, 1, 1, 2] => CFAPattern::RGGB,
            [2, 1, 1, 0] => CFAPattern::BGGR,
            [1, 0, 2, 1] => CFAPattern::GRBG,
            [1, 2, 0, 1] => CFAPattern::GBRG,
            _ => CFAPattern::RGGB,
        };
        Ok(result)
    }
}

#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Decoding error.")]
    RawInfoError(#[from] quickexif::parsed_info::Error),
    #[error("The decoded image size({0}) is invalid due to the width x height = {1}.")]
    InvalidDecodedImageSize(usize, usize),
}
