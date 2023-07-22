#![allow(dead_code)]

//! Process steps:
//! 1. Create EXIF parsing rule.
//! 2. Extract needed EXIF info from the RAW file.
//! 3. Decode RAW image data to `Vec<u16>` with some preprocesses like BlackLevel substraction and color level scaling up, etc.
//! 4. Do demosaicing.
//! 5. Fix WhiteBalance. (Optional)
//! 6. Convert Colorspace to the target one. (Optional)
//! 
//! 
erreport::gen_report_code!();

pub(crate) mod tool;
pub(crate) mod parse;
pub(crate) mod decode;
pub mod data;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Target is none")]
    IsNone,
    #[error("Cast error caused by bytemuck")]
    CastError,
    #[error("Unknown compression type: {0}")]
    UnknownCompression(u16),
    #[error("{0}")]
    Custom(&'static str)
}