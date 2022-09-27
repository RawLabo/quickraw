use crate::RawFileReadingError;
use super::super::data;
use super::*;

pub fn select_decoder(
    file_buffer: &[u8],
    basic_info: quickexif::ParsedInfo,
    only_thumbnail: bool,
) -> Result<(Box<dyn RawDecoder>, [f32; 9]), RawFileReadingError> {
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
            None => *data::CAM_XYZ_MATRIX
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

    macro_rules! get_decoder {
        ($t:ident) => {{
            let task = if only_thumbnail {
                &$t::THUMBNAIL_RULE
            } else {
                &$t::IMAGE_RULE
            };
            let raw_info = quickexif::parse_with_prev_info(file_buffer, &task, basic_info)?;
            $t::General::new(raw_info)
        }};
    }

    let decoder: Result<Box<dyn RawDecoder>, RawFileReadingError> = match dng_version {
        None => match make {
            "NIKON" | "NIKON CORPORATION" => {
                let decoder = get_decoder!(nikon);
                Ok(Box::new(decoder))
            }
            "SONY" => {
                let decoder = get_decoder!(sony);
                Ok(Box::new(decoder))
            }
            "Panasonic" => {
                let decoder = get_decoder!(panasonic);
                Ok(Box::new(decoder))
            }
            "OLYMPUS CORPORATION" | "OLYMPUS IMAGING CORP." => {
                let decoder = get_decoder!(olympus);
                Ok(Box::new(decoder))
            }
            "FUJIFILM" => {
                let decoder = get_decoder!(fujifilm);
                Ok(Box::new(decoder))
            }
            _ => Err(RawFileReadingError::MakerIsNotSupportedYet(make.to_owned())),
        },
        Some(_version) => {
            let decoder = get_decoder!(adobe);
            Ok(Box::new(decoder))
        }
    };

    Ok((decoder?, cam_matrix))
}
