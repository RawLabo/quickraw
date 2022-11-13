#![allow(dead_code)]

use decode::CFAPattern;
use pass::*;

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
    pub fn data(self) -> Vec<u16> {
        self.data
    }
    #[wasm_bindgen(getter)]
    pub fn white_balance(&self) -> Vec<f32> {
        self.white_balance.to_vec()
    }
    #[wasm_bindgen(getter)]
    pub fn color_matrix(&self) -> Vec<f32> {
        self.color_matrix.to_vec()
    }
}

#[wasm_bindgen]
pub fn load_image(input: Vec<u8>) -> Result<Image, JsError> {
    let decoded_image = decode::decode_buffer(input)?;

    let image = decoded_image.image;
    let orientation = decoded_image.orientation as isize;
    let width = decoded_image.width;
    let height = decoded_image.height;

    let iter = image.iter().copied();
    let data = pass::iters_to_vec!(
        iter
            ..enumerate()
            .pixel_info(width, height)
            [decoded_image.cfa_pattern] {
                CFAPattern::RGGB => .linear_rggb(&image, width),
                CFAPattern::GRBG => .linear_grbg(&image, width),
                CFAPattern::GBRG => .linear_gbrg(&image, width),
                CFAPattern::BGGR => .linear_bggr(&image, width),
                CFAPattern::XTrans0 => .linear_xtrans0(&image, width),
                CFAPattern::XTrans1 => .linear_xtrans1(&image, width)
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
pub fn encode_to_jpeg(pixels: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>, JsError> {
    use image::codecs::jpeg;
    use image::ColorType;
    use std::io::Cursor;

    let mut writer = Cursor::new(vec![]);
    let mut encoder = jpeg::JpegEncoder::new_with_quality(&mut writer, 98);
    encoder.encode(&pixels, width, height, ColorType::Rgba8)?;

    Ok(writer.into_inner())
}

#[wasm_bindgen]
pub struct Thumbnail {
    data: Vec<u8>,
    orientation: isize,
}

#[wasm_bindgen]
impl Thumbnail {
    #[wasm_bindgen(getter)]
    pub fn data(self) -> Vec<u8> {
        self.data
    }
}

#[wasm_bindgen]
pub fn load_thumbnail(buffer: Vec<u8>) -> Result<Thumbnail, JsError> {
    let (data, orientation) = export::load_thumbnail(&buffer)?;
    Ok(Thumbnail {
        data,
        orientation: orientation as isize,
    })
}
