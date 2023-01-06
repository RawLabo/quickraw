use super::*;
use std::{fs::File, io::Read};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum CFAPattern {
    RGGB,
    GRBG,
    GBRG,
    BGGR,
    XTrans0, // RBGBRG
    XTrans1, // GGRGGB
}

pub struct Crop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct DecodedImage {
    pub cfa_pattern: CFAPattern,
    pub width: usize,
    pub height: usize,
    pub crop: Option<Crop>,
    pub orientation: Orientation,
    pub image: Vec<u16>,
    pub white_balance: [i32; 3],
    pub cam_matrix: [f32; 9],
    pub parsed_info: quickexif::ParsedInfo,
}

pub enum Orientation {
    Horizontal = 0,
    Rotate90 = 90,
    Rotate180 = 180,
    Rotate270 = 270,
}

pub(super) fn get_buffer_from_file(path: &str) -> Result<Vec<u8>, RawFileReadingError> {
    let mut f =
        File::open(path).map_err(|_| RawFileReadingError::FileNotExisted(path.to_owned()))?;
    let len = f
        .metadata()
        .map_err(|_| RawFileReadingError::FileMetadataReadingError(path.to_owned()))?
        .len() as usize;
    let mut buffer = vec![0u8; len];
    f.read(&mut buffer)
        .map_err(|_| RawFileReadingError::FileContentReadingError(path.to_owned()))?;

    Ok(buffer)
}
fn prepare_buffer(mut buffer: Vec<u8>) -> Vec<u8> {
    buffer.extend([0u8; 16]); // + 16 is for BitPumpMSB fix

    fuji_buffer_fix(buffer)
}
fn fuji_buffer_fix(buffer: Vec<u8>) -> Vec<u8> {
    if buffer[..4] == [0x46, 0x55, 0x4a, 0x49] {
        buffer[148..].to_vec()
    } else {
        buffer
    }
}
fn fuji_buffer_slice_fix(buffer: &[u8]) -> &[u8] {
    if buffer[..4] == [0x46, 0x55, 0x4a, 0x49] {
        &buffer[148..]
    } else {
        buffer
    }
}

/// Gets `RawImage` from a file
#[cfg_attr(not(feature = "wasm-bindgen"), fn_util::bench(decoding))]
pub fn decode_file(path: &str) -> Result<DecodedImage, RawFileReadingError> {
    let buffer = get_buffer_from_file(path)?;
    decode_buffer(buffer)
}

/// Gets `RawImage` from a buffer
#[inline(always)]
pub fn decode_buffer(buffer: Vec<u8>) -> Result<DecodedImage, RawFileReadingError> {
    let buffer = prepare_buffer(buffer);

    let rule = &utility::BASIC_INFO_RULE;
    let decoder_select_info = quickexif::parse(&buffer, rule)?;

    let decoded_image = maker::selector::select_and_decode(buffer.as_slice(), decoder_select_info)?;

    Ok(decoded_image)
}

pub(super) fn get_exif_info(buffer: &[u8]) -> Result<quickexif::ParsedInfo, RawFileReadingError> {
    let buffer = fuji_buffer_slice_fix(buffer);
    let rule = &utility::BASIC_INFO_RULE;
    let decoder_select_info = quickexif::parse(buffer, rule)?;
    let result = maker::selector::select_and_decode_exif_info(buffer, decoder_select_info)?;
    Ok(result)
}

pub(super) fn get_thumbnail(buffer: &[u8]) -> Result<(&[u8], Orientation), RawFileReadingError> {
    let buffer = fuji_buffer_slice_fix(buffer);
    let rule = &utility::BASIC_INFO_RULE;
    let decoder_select_info = quickexif::parse(buffer, rule)?;
    let result = maker::selector::select_and_decode_thumbnail(buffer, decoder_select_info)?;
    Ok(result)
}
