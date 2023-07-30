#![allow(dead_code)]

//! Process steps:
//! 1. Create EXIF parsing rule.
//! 2. Extract needed EXIF info from the RAW file.
//! 3. Decode RAW image data to `Box<[u16]>` with some preprocesses like BlackLevel substraction and color level scaling up, etc.
//! 4. Do demosaicing.
//! 5. Fix WhiteBalance. (Optional)
//! 6. Convert Colorspace to the target one. (Optional)
//!
//!
//!

use std::io::{Read, Seek};

erreport::gen_report_code!();
use parse::{
    base::{detect, Kind},
    ColorMatrix, DecodingInfo,
};
use report::{Report, ToReport};

pub(crate) mod color;
pub(crate) mod decode;
pub(crate) mod demosaicing;
pub(crate) mod parse;
pub(crate) mod tool;

pub use color::data as color_data;

use crate::{
    decode::{Decode, Parse},
    demosaicing::*,
    parse::arw::ArwInfo,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("This raw file is unsupported")]
    UnsupportedRawFile,
    #[error("Target is none")]
    IsNone,
    #[error("Cast error caused by bytemuck")]
    CastError,
    #[error("Unknown compression type: {0}")]
    UnknownCompression(u16),
    #[error("The decoded image size is invalid: {0} * {1} != {2}")]
    InvalidDecodedImage(usize, usize, usize),
    #[error("{0}")]
    Custom(&'static str),
}

fn decode<T: Parse<T> + Decode<T>>(
    mut reader: impl Read + Seek,
) -> Result<(Box<[u16]>, DecodingInfo), Report> {
    let info = T::parse_exif(&mut reader).to_report()?;
    let (strip_addr, strip_size) = info.get_strip_info();
    let strip_bytes = parse::get_bytes(&mut reader, strip_addr, strip_size).to_report()?;
    let image_bytes = info.decode_with_preprocess(strip_bytes)?;
    Ok((image_bytes, info.to_decoding_info()))
}

pub fn extract_image(
    mut reader: impl Read + Seek,
    gamma: f32,
    color_space: &[f32; 9],
) -> Result<(Box<[u16]>, usize, usize), Report> {
    // parse and decode
    let (kind, model) = detect(&mut reader).to_report()?;
    let (image_bytes, info): (Box<[u16]>, DecodingInfo) = match kind {
        Kind::Arw => decode::<ArwInfo>(&mut reader).to_report()?,
        _ => return Err(Error::UnsupportedRawFile).to_report(),
    };

    // safety check
    let w = info.width;
    let h = info.height;
    if w * h != image_bytes.len() {
        return Err(Error::InvalidDecodedImage(w, h, image_bytes.len())).to_report();
    }

    // prepare color conversion
    let gamma_lut = color::gen_gamma_lut(gamma);
    let mut color_matrix: ColorMatrix = color::data::CAM_XYZ_MAP
        .get(&model)
        .ok_or(Error::IsNone)
        .to_report()?
        .into();
    color_matrix.update_colorspace(color_space);

    // demosaicing and postprocesses
    let mut image = vec![0u16; image_bytes.len() * 3];
    let mut pixel_info = PixelInfo::new(w, h);
    macro_rules! gen_cfa_processing_branch {
        ($method:expr) => {
            for (i, v) in image.chunks_exact_mut(3).enumerate() {
                let stat = pixel_info.get_stat_and_update();
                let rgb = $method(i, w, stat, &image_bytes);
                let rgb = info.white_balance.fix(rgb);
                let rgb = color_matrix.shift_color(&rgb);
                let rgb = color::gamma_correct(rgb, &gamma_lut);
                v.copy_from_slice(&rgb);
            }
        };
    }
    match info.cfa_pattern {
        parse::CFAPattern::Rggb => {
            gen_cfa_processing_branch!(demosaicing::linear::rggb)
        }
        _ => {}
    }

    Ok((image.into_boxed_slice(), w, h))
}
