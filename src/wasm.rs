#![allow(dead_code)]

use std::io::Cursor;

use decode::CFAPattern;
use pass::*;

use crate::{
    decode::Orientation,
    utility::{log2, ArrayMulNum},
};

use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub orientation: isize,
    data: Vec<u16>,
    white_balance: [f32; 3],
    color_matrix: [f32; 9],
}

#[wasm_bindgen]
impl Image {
    #[wasm_bindgen(getter)]
    pub fn white_balance(&self) -> Vec<f32> {
        self.white_balance.to_vec()
    }
    #[wasm_bindgen(getter)]
    pub fn color_matrix(&self) -> Vec<f32> {
        self.color_matrix.to_vec()
    }
    #[wasm_bindgen(getter)]
    pub fn data(self) -> Vec<u16> {
        self.data
    }
}

fn expand_err<T, E: Into<anyhow::Error>>(input: Result<T, E>) -> Result<T, JsError> {
    input.map_err(|e| JsError::new(&format!("{:?}", anyhow::anyhow!(e))))
}

macro_rules! gen_image_loader {
    ($name:ident, $rggb:ident, $grbg:ident, $gbrg:ident, $bggr:ident) => {
        #[wasm_bindgen]
        pub fn $name(input: Vec<u8>) -> Result<Image, JsError> {
            let decoded_image = expand_err(decode::decode_buffer(input))?;

            let image = decoded_image.image;
            let orientation = decoded_image.orientation as isize;
            let width = decoded_image.width;
            let height = decoded_image.height;

            let iter = image.iter().copied();
            let data = pass::iters_to_vec!(
                iter
                    ..enumerate()
                    [decoded_image.cfa_pattern] {
                        CFAPattern::RGGB => .$rggb(&image, width, height),
                        CFAPattern::GRBG => .$grbg(&image, width, height),
                        CFAPattern::GBRG => .$gbrg(&image, width, height),
                        CFAPattern::BGGR => .$bggr(&image, width, height),
                        CFAPattern::XTrans0 => .linear_xtrans0(&image, width, height),
                        CFAPattern::XTrans1 => .linear_xtrans1(&image, width, height)
                    }
                    ..flatten()
            );

            let color_matrix = utility::matrix3_mul(&data::XYZ2SRGB, &decoded_image.cam_matrix);
            let white_balance = {
                let [r, g, b] = decoded_image.white_balance;
                [r as f32 / g as f32, 1f32, b as f32 / g as f32]
            };

            Ok(Image {
                data,
                orientation,
                width,
                height,
                white_balance,
                color_matrix,
            })
        }
    };
}
gen_image_loader!(
    load_image,
    linear_rggb,
    linear_grbg,
    linear_gbrg,
    linear_bggr
);
gen_image_loader!(
    load_image_enhanced,
    elinear_rggb,
    elinear_grbg,
    elinear_gbrg,
    elinear_bggr
);

#[wasm_bindgen]
pub fn calc_histogram(pixels: Vec<u8>) -> Vec<u32> {
    let mut histogram = [0u32; 256 * 3 + 1];
    let mut max = 0u32;

    for point in pixels.chunks_exact(4) {
        if let &[r, g, b, _] = point {
            let r_index = r as usize;
            let g_index = 256 + g as usize;
            let b_index = 256 * 2 + b as usize;

            max = [r_index, g_index, b_index]
                .into_iter()
                .fold(max, |acc, index| {
                    histogram[index] += 1;
                    acc.max(histogram[index])
                });
        }
    }

    if let Some(last) = histogram.last_mut() {
        *last = max;
    }

    histogram.to_vec()
}

#[cfg(feature = "image")]
#[wasm_bindgen]
pub fn encode_to_jpeg(
    pixels_ptr: *mut u8,
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, JsError> {
    use image::codecs::jpeg;
    use image::ColorType;

    let len = (width * height * 4) as usize;
    let pixels = unsafe { Vec::from_raw_parts(pixels_ptr, len, len) };

    let mut writer = Cursor::new(vec![]);
    let mut encoder = jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
    expand_err(encoder.encode(&pixels, width, height, ColorType::Rgba8))?;

    Ok(writer.into_inner())
}

#[wasm_bindgen]
pub struct ExifWithThumbnail {
    pub orientation: isize,
    exif: String,
    thumbnail: Vec<u8>,
}

#[wasm_bindgen]
impl ExifWithThumbnail {
    #[wasm_bindgen(getter)]
    pub fn thumbnail(self) -> Vec<u8> {
        self.thumbnail
    }
    #[wasm_bindgen(getter)]
    pub fn exif(&self) -> String {
        self.exif.clone()
    }
}

fn quick_image_load(
    input: Vec<u8>,
) -> Result<(Vec<u8>, u32, u32, Orientation), RawFileReadingError> {
    let decoded_image = decode::decode_buffer(input)?;

    let width = decoded_image.width;
    let image = decoded_image
        .image
        .chunks(width * 4)
        .enumerate()
        .map(|(row_index, row)| {
            let left_bound = row_index % 4 * width;
            row[left_bound..left_bound + width]
                .chunks(4)
                .enumerate()
                .map(|(column_index, values)| values[column_index % 4])
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    let orientation = decoded_image.orientation;
    let width = decoded_image.width / 4;
    let height = decoded_image.height / 4;

    let gamma_lut = gen_gamma_lut(0.45);
    let color_matrix = utility::matrix3_mul(&data::XYZ2SRGB, &decoded_image.cam_matrix);
    let color_matrix = color_matrix.mul(1 << BIT_SHIFT);
    let white_balance = decoded_image
        .white_balance
        .mul(1 << (BIT_SHIFT - log2(decoded_image.white_balance[1])));

    let iter = image.iter().copied();
    let data = pass::iters_to_vec!(
        iter
            ..enumerate()
            [decoded_image.cfa_pattern] {
                CFAPattern::RGGB => .linear_rggb(&image, width, height),
                CFAPattern::GRBG => .linear_grbg(&image, width, height),
                CFAPattern::GBRG => .linear_gbrg(&image, width, height),
                CFAPattern::BGGR => .linear_bggr(&image, width, height),
                CFAPattern::XTrans0 => .linear_xtrans0(&image, width, height),
                CFAPattern::XTrans1 => .linear_xtrans1(&image, width, height)
            }
            .u16rgb_to_i32rgb()
            .white_balance_fix(&white_balance)
            .color_convert(&color_matrix)
            .gamma_correct(&gamma_lut)
            .i32rgb_to_u8rgb()
            ..flatten()
    );

    Ok((data, width as u32, height as u32, orientation))
}

#[wasm_bindgen]
pub fn load_exif_with_thumbnail(buffer: Vec<u8>) -> Result<ExifWithThumbnail, JsError> {
    let info = expand_err(export::load_exif(&buffer))?;
    let exif = info.stringify_all()?;
    let (thumbnail, orientation) = match export::load_thumbnail(&buffer) {
        Ok(x) => x,
        Err(_) => {
            use image::codecs::jpeg;
            use image::ColorType;

            let (data, width, height, orientation) = quick_image_load(buffer)?;
            let mut writer = Cursor::new(vec![]);
            let mut encoder = jpeg::JpegEncoder::new_with_quality(&mut writer, 80);
            expand_err(encoder.encode(&data, width, height, ColorType::Rgb8))?;

            (writer.into_inner(), orientation)
        }
    };

    Ok(ExifWithThumbnail {
        orientation: orientation as isize,
        thumbnail,
        exif,
    })
}
