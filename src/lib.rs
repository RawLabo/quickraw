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

pub fn extract_image(
    mut reader: impl Read + Seek,
    gamma: f32,
    color_space: &[f32; 9],
) -> Result<(Box<[u16]>, usize, usize), Report> {
    let (kind, model) = detect(&mut reader).to_report()?;
    let (image_bytes, info): (Box<[u16]>, DecodingInfo) = match kind {
        Kind::Arw => {
            let info = parse::arw::parse_exif(&mut reader).to_report()?;
            let strip_bytes =
                parse::get_bytes(&mut reader, info.strip_addr, info.strip_size).to_report()?;
            let image_bytes = decode::arw::decode_with_preprocess(&info, strip_bytes)?;
            (image_bytes, info.into())
        }
        _ => return Err(Error::UnsupportedRawFile).to_report(),
    };

    // prepare color conversion
    let gamma_lut = color::gen_gamma_lut(gamma);
    let mut color_matrix: ColorMatrix = color::data::CAM_XYZ_MAP
        .get(&model)
        .ok_or(Error::IsNone)
        .to_report()?
        .into();
    color_matrix.update_colorspace(&color_space);

    let w = info.width;
    let h = info.height;

    if w * h != image_bytes.len() {
        return Err(Error::InvalidDecodedImage(w, h, image_bytes.len())).to_report();
    }

    let mut image = vec![0u16; image_bytes.len() * 3];
    let linear_demosaicing = info.cfa_pattern.linear_method();

    image.chunks_exact_mut(3).enumerate().for_each(|(i, v)| {
        let rgb = linear_demosaicing(i, w, h, &image_bytes);
        let rgb = info.white_balance.fix(rgb);
        let rgb = color_matrix.shift_color(&rgb);
        let rgb = color::gamma_correct(rgb, &gamma_lut);
        v.copy_from_slice(&rgb);
    });

    Ok((image.into_boxed_slice(), w, h))
}