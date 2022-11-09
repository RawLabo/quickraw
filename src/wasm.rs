#![allow(dead_code)]

use crate::utility::ArrayMulNum;

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
    pub rotation: isize,
    data: Vec<u16>,
    wb: [f32; 3],
    color_matrix: [f32; 9],
}

#[wasm_bindgen]
impl Image {
    #[wasm_bindgen(getter)]
    pub fn data(self) -> Vec<u16> {
        self.data
    }
    #[wasm_bindgen(getter)]
    pub fn wb(&self) -> Vec<f32> {
        self.wb.to_vec()
    }
    #[wasm_bindgen(getter)]
    pub fn color_matrix(&self) -> Vec<f32> {
        self.color_matrix.to_vec()
    }
}

#[wasm_bindgen]
pub fn load_image(input: Vec<u8>) -> Image {
    use pass::*;

    let decoded_image = decode::new_image_from_buffer(input).unwrap();

    let color_space = data::XYZ2SRGB;
    let color_space = utility::matrix3_mul(&color_space, &decoded_image.cam_matrix);

    let color_matrix = color_space;

    // let color_space = color_space.mul(1 << BIT_SHIFT);

    let wb = [
        decoded_image.white_balance[0] as f32 / decoded_image.white_balance[1] as f32,
        1f32,
        decoded_image.white_balance[2] as f32 / decoded_image.white_balance[1] as f32,
    ];

    // let white_balance = decoded_image
    //     .white_balance
    //     .mul(1 << (BIT_SHIFT - utility::log2(decoded_image.white_balance[1])));

    // let gamma_lut = gen_gamma_lut(0.45);

    let image = decoded_image.image;
    let rotation = decoded_image.orientation as isize;
    let width = decoded_image.width;
    let height = decoded_image.height;

    let iter = image.iter().copied();
    let data = pass::iters_to_vec! (
        iter
            ..enumerate()
            .pixel_info(width, height)
            .demosaic(&image, width)
            // .u16rgb_to_i32rgb()
            // .sub_black_level(decoded_image.black_level)
            // .level_scale_up(decoded_image.scale_factor)
            // .white_balance_fix(&white_balance)
            // .color_convert(&color_space)
            // .gamma_correct(&gamma_lut)
            ..flatten()
    );

    Image {
        data,
        rotation,
        width,
        height,
        wb,
        color_matrix,
    }
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
pub fn encode_to_jpeg(pixels: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
    use image::codecs::jpeg;
    use image::ColorType;
    use std::io::Cursor;

    let mut writer = Cursor::new(vec![]);
    let mut encoder = jpeg::JpegEncoder::new_with_quality(&mut writer, 98);
    encoder
        .encode(&pixels, width, height, ColorType::Rgba8)
        .unwrap();

    writer.into_inner()
}
