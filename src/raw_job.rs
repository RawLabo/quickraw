use std::{fs::File, io::Read};

use crate::raw::Orientation;

use super::{maker::*, *};

impl RawJob {
    pub fn get_buffer_from_file(path: &str) -> Result<Vec<u8>, RawFileReadingError> {
        let mut f =
            File::open(path).map_err(|_| RawFileReadingError::FileNotExisted(path.to_owned()))?;
        let len = f
            .metadata()
            .map_err(|_| RawFileReadingError::FileMetadataReadingError(path.to_owned()))?
            .len() as usize;
        let mut buffer = vec![0u8; len + 16]; // + 16 is for BitPumpMSB fix
        f.read(&mut buffer)
            .map_err(|_| RawFileReadingError::FileContentReadingError(path.to_owned()))?;

        if &buffer[..4] == [0x46, 0x55, 0x4a, 0x49] {
            // fuji raw fix
            Ok(buffer.drain(148..).collect())
        } else {
            Ok(buffer)
        }
    }

    #[attrs::bench(loading_with_decoder_choosing)]
    pub fn new(path: &str) -> Result<RawJob, RawFileReadingError> {
        let buffer = RawJob::get_buffer_from_file(path)?;
        RawJob::new_from_buffer(buffer)
    }

    pub fn new_from_buffer(buffer: Vec<u8>) -> Result<RawJob, RawFileReadingError> {
        let rule = &maker::utility::BASIC_INFO_RULE;
        let decoder_select_info = quickexif::parse(&buffer, rule)?;

        let (decoder, cam_matrix) =
            selector::select_decoder(buffer.as_slice(), decoder_select_info, false)?;
        let white_balance = decoder.get_white_balance()?;

        let raw_job = RawJob {
            file_buffer: buffer,
            decoder,
            white_balance,
            cam_matrix,
        };

        Ok(raw_job)
    }

    pub fn get_thumbnail<'a>(
        buffer: &'a [u8],
    ) -> Result<(&'a [u8], Orientation), RawFileReadingError> {
        let rule = &maker::utility::BASIC_INFO_RULE;
        let decoder_select_info = quickexif::parse(&buffer, rule)?;

        let (decoder, _) = selector::select_decoder(buffer, decoder_select_info, true)?;

        let result = decoder.get_thumbnail(buffer)?;

        Ok((result, decoder.get_orientation()))
    }
}
