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
//!

use std::fs::File;

erreport::gen_report_code!();
use parse::ColorMatrix;
use report::{Report, ToReport};

pub(crate) mod color;
pub mod data;
pub(crate) mod decode;
pub(crate) mod demosaicing;
pub(crate) mod parse;
pub(crate) mod tool;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Target is none")]
    IsNone,
    #[error("Cast error caused by bytemuck")]
    CastError,
    #[error("Unknown compression type: {0}")]
    UnknownCompression(u16),
    #[error("{0}")]
    Custom(&'static str),
}

pub fn extract_image(path: &str) -> Result<(Box<[u16]>, usize, usize), Report> {
    let mut file_reader = File::open(path).to_report()?;
    let info = parse::arw::parse_exif(&mut file_reader).to_report()?;
    let strip_bytes =
        parse::get_bytes(&mut file_reader, info.strip_addr, info.strip_size).to_report()?;
    let image_bytes = decode::arw::decode_with_preprocess(&info, strip_bytes)?;

    let gamma_lut = color::gen_gamma_lut(1f32 / 2.2);
    let mut color_matrix: ColorMatrix = data::CAM_XYZ_MAP.get("ILCE-7C").unwrap().into();
    color_matrix.update_colorspace(&data::XYZ2SRGB);

    let w = info.width;
    let h = info.height;
    let image: Box<_> = image_bytes
        .iter()
        .enumerate()
        .flat_map(|(i, v)| {
            let rgb = demosaicing::linear::rggb(i, w, h, *v, &image_bytes);
            let rgb = info.white_balance.fix(rgb);
            let rgb = color_matrix.shift_color(rgb);
            color::gamma_correct(rgb, &gamma_lut)
        })
        .collect();

    Ok((image, w, h))
}
