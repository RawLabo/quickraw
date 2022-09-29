use std::{fs::File, io::Read};

use raw::{Orientation, RawImage};

use super::*;

pub(super) fn get_buffer_from_file(path: &str) -> Result<Vec<u8>, RawFileReadingError> {
    let mut f =
        File::open(path).map_err(|_| RawFileReadingError::FileNotExisted(path.to_owned()))?;
    let len = f
        .metadata()
        .map_err(|_| RawFileReadingError::FileMetadataReadingError(path.to_owned()))?
        .len() as usize;
    let mut buffer = vec![0u8; len + 16]; // + 16 is for BitPumpMSB fix
    f.read(&mut buffer)
        .map_err(|_| RawFileReadingError::FileContentReadingError(path.to_owned()))?;

    if buffer[..4] == [0x46, 0x55, 0x4a, 0x49] {
        // fuji raw fix
        Ok(buffer.drain(148..).collect())
    } else {
        Ok(buffer)
    }
}

/// Gets `RawImage` from a file
#[fn_util::bench(decoding)]
pub fn new_image_from_file(path: &str) -> Result<RawImage, RawFileReadingError> {
    let buffer = get_buffer_from_file(path)?;
    new_image_from_buffer(buffer)
}

/// Gets `RawImage` from a buffer
pub fn new_image_from_buffer(buffer: Vec<u8>) -> Result<RawImage, RawFileReadingError> {
    let rule = &utility::BASIC_INFO_RULE;
    let decoder_select_info = quickexif::parse(&buffer, rule)?;

    let raw_image =
        maker::selector::select_and_decode(buffer.as_slice(), decoder_select_info)?;

    Ok(raw_image)
}

pub(super) fn get_exif_info(buffer: &[u8]) -> Result<quickexif::ParsedInfo, RawFileReadingError> {
    let rule = &utility::BASIC_INFO_RULE;
    let decoder_select_info = quickexif::parse(buffer, rule)?;
    let result = maker::selector::select_and_decode_exif_info(buffer, decoder_select_info)?;
    Ok(result)
}

pub(super) fn get_thumbnail(buffer: &[u8]) -> Result<(&[u8], Orientation), RawFileReadingError> {
    let rule = &utility::BASIC_INFO_RULE;
    let decoder_select_info = quickexif::parse(buffer, rule)?;

    let result = maker::selector::select_and_decode_thumbnail(buffer, decoder_select_info)?;

    Ok(result)
}
