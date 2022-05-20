use crate::{
    raw::{CFAPattern, Crop, Orientation},
    tiff::{ExifTask, ParsedRawInfo, RawInfoError},
};
use thiserror::Error;

#[macro_use]
mod macros;
pub mod selector;
pub mod utility;

pub mod adobe;
mod cam_matrix;
mod decode_utility;
pub mod fujifilm;
pub mod nikon;
pub mod olympus;
pub mod panasonic;
pub mod sony;

pub trait RawDecoder {
    fn new(info: ParsedRawInfo) -> Self
    where
        Self: Sized;
    fn get_task(only_thumbnail: bool, model: String) -> ExifTask
    where
        Self: Sized;
    fn get_info(&self) -> &ParsedRawInfo;
    fn get_white_balance(&self) -> Result<[i32; 3], DecodingError> {
        let info = self.get_info();
        (|| -> Result<[i32; 3], DecodingError> {
            Ok([
                info.i32("white_balance_r")?,
                info.i32("white_balance_g")?,
                info.i32("white_balance_b")?,
            ])
        })()
        .map_err(|_| DecodingError::GetWhiteBalanceError)
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

    fn pre_process(&self, buffer: &[u8]) -> Result<Vec<u16>, DecodingError> {
        self.inner_pre_process(buffer).map_err(|err|DecodingError::PreProcessError(Box::new(err)))
    }
    fn get_cfa_pattern(&self) -> Result<CFAPattern, DecodingError> {
        self.inner_get_cfa_pattern().map_err(|err|DecodingError::GetCFAPatternError(Box::new(err)))
    }
    fn get_thumbnail<'a>(&self, buffer: &'a [u8]) -> Result<&'a [u8], DecodingError> {
        self.inner_get_thumbnail(buffer).map_err(|err|DecodingError::GetThumbnailError(Box::new(err)))
    }

    fn inner_pre_process(&self, buffer: &[u8]) -> Result<Vec<u16>, DecodingError>;
    fn inner_get_cfa_pattern(&self) -> Result<CFAPattern, DecodingError> {
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
    fn inner_get_thumbnail<'a>(&self, buffer: &'a [u8]) -> Result<&'a [u8], DecodingError>;
}

#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Decoding error.")]
    RawInfoError(#[from] RawInfoError),
    #[error("Pre process error. This may caused by a decoding issue.")]
    PreProcessError(#[source] Box<DecodingError>),
    #[error("Cannot get the white balance of the raw file.")]
    GetWhiteBalanceError,
    #[error("Cannot get CFA pattern of the raw file.")]
    GetCFAPatternError(#[source] Box<DecodingError>),
    #[error("Cannot get the thumbnail image of the raw file.")]
    GetThumbnailError(#[source] Box<DecodingError>),
    #[error("The decoded image size({0}) is invalid due to the width x height = {1}.")]
    InvalidDecodedImageSize(usize, usize),
}
