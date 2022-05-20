use crate::{
    tiff::{ParsedRawInfo, Parser},
    RawFileReadingError,
};

use super::*;

pub fn select_decoder(
    file_buffer: &[u8],
    basic_info: ParsedRawInfo,
    only_thumbnail: bool,
) -> Result<(Box<dyn RawDecoder>, [f32; 9]), RawFileReadingError> {
    let make = basic_info
        .str("make")
        .map_err(|_| RawFileReadingError::CannotReadMake)?;
    let model = basic_info
        .str("model")
        .map_err(|_|RawFileReadingError::CannotReadModel)?
        .split_whitespace()
        .collect::<String>();

    let dng_version = basic_info.u16("dng_version").ok();

    let cam_matrix = if only_thumbnail {
        [0f32;9]
    } else {
        match dng_version {
            None => {
                *cam_matrix::CAM_XYZ_MATRIX
                .get(model.as_str())
                .ok_or_else(|| RawFileReadingError::ModelIsNotSupportedYet(model.clone()))?
            },
            Some(_) => {
                let mut matrix = [0f32;9];
                for i in 0..9 {
                    matrix[i] = basic_info.f64(format!("c{}", i.to_string()).as_str())? as f32;
                }
                utility::matrix3_inverse(&mut matrix);
                utility::matrix3_normalize(&mut matrix);
                matrix
            }
        }
    };

    macro_rules! get_decoder {
        ($t:ty) => {{
            let task = <$t>::get_task(only_thumbnail, model);
            let raw_info = Parser::get_raw_info_with_content(file_buffer, &task, basic_info.content)?;
            <$t>::new(raw_info)
        }};
    }

    let decoder: Result<Box<dyn RawDecoder>, RawFileReadingError> = match dng_version {
        None => match make {
            "NIKON" | "NIKON CORPORATION" => {
                let decoder = get_decoder!(nikon::NikonGeneral);
                Ok(Box::new(decoder))
            }
            "SONY" => {
                let decoder = get_decoder!(sony::SonyGeneral);
                Ok(Box::new(decoder))
            }
            "Panasonic" => {
                let decoder = get_decoder!(panasonic::PanasonicGeneral);
                Ok(Box::new(decoder))
            }
            "OLYMPUS CORPORATION" | "OLYMPUS IMAGING CORP." => {
                let decoder = get_decoder!(olympus::OlympusGeneral);
                Ok(Box::new(decoder))
            }
            "FUJIFILM" => {
                let decoder = get_decoder!(fujifilm::FujifilmGeneral);
                Ok(Box::new(decoder))
            }
            _ => Err(RawFileReadingError::MakerIsNotSupportedYet(make.to_owned())),
        },
        Some(_version) => {
            let decoder = get_decoder!(adobe::AdobeGeneral);
            Ok(Box::new(decoder))
        }
    };

    Ok((decoder?, cam_matrix))
}
