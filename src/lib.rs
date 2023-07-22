#![allow(dead_code)]

//! Process steps:
//! 1. Create EXIF parsing rule.
//! 2. Extract needed EXIF info from the RAW file.
//! 3. Decode RAW image data to `Vec<u16>`.
//! 4. Do preprocesses like BlackLevel substraction and color level scaling up, etc.
//! 5. Do demosaicing.
//! 6. Fix WhiteBalance. (Optional)
//! 7. Convert Colorspace to the target one. (Optional)
//! 
//! 
erreport::gen_report_code!();

pub(crate) mod parse;
pub(crate) mod tool;
pub mod data;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("target is none")]
    IsNone,
    #[error("{0}")]
    Custom(&'static str)
}