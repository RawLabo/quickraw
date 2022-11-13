use super::super::data;
use super::*;
use crate::decode::DecodedImage;
use crate::RawFileReadingError;

fn prepare(
    basic_info: &quickexif::ParsedInfo,
    only_thumbnail: bool,
) -> Result<(&str, Option<u16>, [f32; 9]), RawFileReadingError> {
    let make = basic_info
        .str("make")
        .map_err(|_| RawFileReadingError::CannotReadMake)?;
    let model = basic_info
        .str("model")
        .map_err(|_| RawFileReadingError::CannotReadModel)?
        .split_whitespace()
        .collect::<String>();

    let dng_version = basic_info.u16("dng_version").ok();

    let cam_matrix = if only_thumbnail {
        [0f32; 9]
    } else {
        match dng_version {
            None => *data::CAM_XYZ_MAP
                .get(model.as_str())
                .ok_or_else(|| RawFileReadingError::ModelIsNotSupportedYet(model.clone()))?,
            Some(_) => {
                let mut matrix = [0f32; 9];
                for (i, item) in matrix.iter_mut().enumerate() {
                    *item = basic_info.f64(format!("c{}", i).as_str())? as f32;
                }
                utility::matrix3_inverse(&mut matrix);
                utility::matrix3_normalize(&mut matrix);
                matrix
            }
        }
    };

    Ok((make, dng_version, cam_matrix))
}

pub(in super::super) fn select_and_decode_exif_info(
    file_buffer: &[u8],
    basic_info: quickexif::ParsedInfo,
) -> Result<quickexif::ParsedInfo, RawFileReadingError> {
    let (make, dng_version, _) = prepare(&basic_info, true)?;

    let rule = match dng_version {
        None => match make {
            "NIKON" | "NIKON CORPORATION" => Ok(&nikon::IMAGE_RULE),
            "SONY" => Ok(&sony::IMAGE_RULE),
            "Panasonic" => Ok(&panasonic::IMAGE_RULE),
            "OLYMPUS CORPORATION" | "OLYMPUS IMAGING CORP." => Ok(&olympus::IMAGE_RULE),
            "FUJIFILM" => Ok(&fujifilm::IMAGE_RULE),
            _ => Err(RawFileReadingError::MakerIsNotSupportedYet(make.to_owned())),
        },
        Some(_version) => Ok(&adobe::IMAGE_RULE),
    }?;

    Ok(quickexif::parse_with_prev_info(
        file_buffer,
        rule,
        basic_info,
    )?)
}

pub(in super::super) fn select_and_decode_thumbnail(
    file_buffer: &[u8],
    basic_info: quickexif::ParsedInfo,
) -> Result<(&[u8], Orientation), RawFileReadingError> {
    let (make, dng_version, _) = prepare(&basic_info, true)?;

    macro_rules! decode {
        ($t:ident) => {{
            let raw_info =
                quickexif::parse_with_prev_info(file_buffer, &$t::THUMBNAIL_RULE, basic_info)?;
            let decoder = $t::General::new(raw_info);
            let thumbnail = decoder.get_thumbnail(&file_buffer)?;
            let orientation = decoder.get_orientation();
            (thumbnail, orientation)
        }};
    }

    match dng_version {
        None => match make {
            "NIKON" | "NIKON CORPORATION" => Ok(decode!(nikon)),
            "SONY" => Ok(decode!(sony)),
            "Panasonic" => Ok(decode!(panasonic)),
            "OLYMPUS CORPORATION" | "OLYMPUS IMAGING CORP." => Ok(decode!(olympus)),
            "FUJIFILM" => Ok(decode!(fujifilm)),
            _ => Err(RawFileReadingError::MakerIsNotSupportedYet(make.to_owned())),
        },
        Some(_version) => Ok(decode!(adobe)),
    }
}

pub(in super::super) fn select_and_decode(
    file_buffer: &[u8],
    basic_info: quickexif::ParsedInfo,
) -> Result<DecodedImage, RawFileReadingError> {
    let (make, dng_version, cam_matrix) = prepare(&basic_info, false)?;

    macro_rules! decode {
        ($t:ident) => {{
            let raw_info =
                quickexif::parse_with_prev_info(file_buffer, &$t::IMAGE_RULE, basic_info)?;
            let width = raw_info.usize("width")?;
            let height = raw_info.usize("height")?;
            let black_level = raw_info.u16("black_level")?;

            let decoder = $t::General::new(raw_info);
            let cfa_pattern = decoder.get_cfa_pattern()?;
            let crop = decoder.get_crop();
            let orientation = decoder.get_orientation();
            let white_balance = decoder.get_white_balance()?;
            let image = decoder.decode_with_preprocess(file_buffer)?;
            let scale_factor = decoder.get_bps_scale()?;

            DecodedImage {
                image,
                width,
                height,
                cfa_pattern,
                crop,
                orientation,
                white_balance,
                cam_matrix,
                black_level,
                scale_factor,
                parsed_info: decoder.into_info()
            }
        }};
    }

    let decoded_image = match dng_version {
        None => match make {
            "NIKON" | "NIKON CORPORATION" => Ok(decode!(nikon)),
            "SONY" => Ok(decode!(sony)),
            "Panasonic" => Ok(decode!(panasonic)),
            "OLYMPUS CORPORATION" | "OLYMPUS IMAGING CORP." => Ok(decode!(olympus)),
            "FUJIFILM" => Ok(decode!(fujifilm)),
            _ => Err(RawFileReadingError::MakerIsNotSupportedYet(make.to_owned())),
        },
        Some(_version) => Ok(decode!(adobe)),
    }?;

    Ok(decoded_image)
}
