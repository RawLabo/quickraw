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
    data: Vec<u16>,
    wb: [f32;3],
    color_matrix: [f32;9]
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

    let color_space = color_space.mul(1 << BIT_SHIFT);

    let wb = [
        decoded_image
        .white_balance[0] as f32 / decoded_image
        .white_balance[1] as f32,
        1f32,
        decoded_image
        .white_balance[2] as f32 / decoded_image
        .white_balance[1] as f32
    ];

    let white_balance = decoded_image
        .white_balance
        .mul(1 << (BIT_SHIFT - utility::log2(decoded_image.white_balance[1])));
    
    // let gamma_lut = gen_gamma_lut(0.45);

    let image = decoded_image.image;
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
        width,
        height,
        wb,
        color_matrix
    }
}

// use crate::{raw::PixelInfo, utility::ArrayMulNum};
// #[wasm_bindgen]
// pub fn load_image(input: Vec<u8>) -> Image {
//     let export = Export::new(
//         Input::ByBuffer(input),
//         Output::new(
//             DemosaicingMethod::Linear,
//             data::XYZ2SRGB,
//             data::GAMMA_LINEAR,
//             OutputType::Raw8,
//             false,
//             false,
//         ),
//     )
//     .unwrap();

//     let (data, width, height) = export.export_8bit_image();
//     Image {
//         data,
//         width,
//         height,
//     }
// }

// #[inline(always)]
// fn get_pixel(image: &[u16], i: usize) -> i32 {
//     unsafe { *image.get_unchecked(i) as i32 }
// }
// #[inline(always)]
// fn avg<const N: usize>(image: &[u16], indexes: [usize; N]) -> i32 {
//     indexes.iter().map(|&i| get_pixel(image, i)).sum::<i32>() / N as i32
// }

// #[inline(always)]
// fn rggb(
//     PixelInfo {
//         i,
//         v,
//         x: _,
//         y: _,
//         is_top,
//         is_left,
//         is_bottom,
//         is_right,
//         is_column_even,
//         is_row_even,
//     }: PixelInfo,
//     image: &[u16],
//     w: usize,
// ) -> (i32, i32, i32) {
//     match (
//         is_top,
//         is_bottom,
//         is_left,
//         is_right,
//         is_column_even,
//         is_row_even,
//     ) {
//         // top left corner
//         (true, _, true, _, _, _) => (v, avg(image, [i + 1, i + w]), get_pixel(image, i + w + 1)),
//         // top right corner
//         (true, _, _, true, _, _) => (get_pixel(image, i - 1), v, get_pixel(image, i + w)),
//         // bottom left corner
//         (_, true, true, _, _, _) => (get_pixel(image, i - w), v, get_pixel(image, i + 1)),
//         // bottom right corner
//         (_, true, _, true, _, _) => (get_pixel(image, i - w - 1), avg(image, [i - w, i - 1]), v),
//         // top edge
//         (true, _, _, _, true, _) => (
//             v,
//             avg(image, [i - 1, i + w, i + 1]),
//             avg(image, [i + w - 1, i + w + 1]),
//         ),
//         (true, _, _, _, false, _) => (avg(image, [i - 1, i + 1]), v, get_pixel(image, i + w)),
//         // bottom edge
//         (_, true, _, _, true, _) => (get_pixel(image, i - w), v, avg(image, [i - 1, i + 1])),
//         (_, true, _, _, false, _) => (get_pixel(image, i - w - 1), avg(image, [i - w, i - 1]), v),
//         // left edge
//         (_, _, true, _, _, true) => (
//             v,
//             avg(image, [i - w, i + 1, i + w]),
//             avg(image, [i - w + 1, i + w + 1]),
//         ),
//         (_, _, true, _, _, false) => (avg(image, [i - w, i + w]), v, get_pixel(image, i + 1)),
//         // right edge
//         (_, _, _, true, _, true) => (get_pixel(image, i - 1), v, get_pixel(image, i + w)),
//         (_, _, _, true, _, false) => (
//             avg(image, [i - w - 1, i + w - 1]),
//             avg(image, [i - w, i + w, i - 1]),
//             v,
//         ),
//         // red
//         (_, _, _, _, true, true) => (
//             v,
//             avg(image, [i - w, i + w, i - 1, i + 1]),
//             avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
//         ),
//         // green1
//         (_, _, _, _, false, true) => (avg(image, [i - 1, i + 1]), v, avg(image, [i - w, i + w])),
//         // green2
//         (_, _, _, _, true, false) => (avg(image, [i - w, i + w]), v, avg(image, [i - 1, i + 1])),
//         // blue
//         (_, _, _, _, false, false) => (
//             avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
//             avg(image, [i - w, i + w, i - 1, i + 1]),
//             v,
//         ),
//     }
// }

// fn pixel_info(
//     iter: impl Iterator<Item = (usize, u16)>,
//     width: usize,
//     height: usize,
// ) -> impl Iterator<Item = PixelInfo> {
//     iter.map(move |(i, v)| PixelInfo::new(i, v, width, height))
// }

// fn demosaic<'a>(
//     iter: impl Iterator<Item = PixelInfo> + 'a,
//     image: &'a [u16],
//     width: usize,
// ) -> impl Iterator<Item = (i32, i32, i32)> + 'a {
//     iter.map(move |pi| rggb(pi, image, width))
// }

// fn color_fix<'a>(
//     iter: impl Iterator<Item = (i32, i32, i32)> + 'a,
//     cc: &'a ColorConversion,
// ) -> impl Iterator<Item = [u16; 3]> + 'a {
//     iter.map(|rgb| cc.convert(rgb))
// }

// fn to8bit(iter: impl Iterator<Item = u16>) -> impl Iterator<Item = u8> {
//     iter.map(|v| (v / 256) as u8)
// }

// fn black_level(
//     iter: impl Iterator<Item = (i32, i32, i32)>,
//     black_level: i32,
// ) -> impl Iterator<Item = (i32, i32, i32)> {
//     iter.map(move |(r, g, b)| {
//         (
//             r.saturating_sub(black_level),
//             g.saturating_sub(black_level),
//             b.saturating_sub(black_level),
//         )
//     })
// }

// fn white_scale(
//     iter: impl Iterator<Item = (i32, i32, i32)>,
// ) -> impl Iterator<Item = (i32, i32, i32)> {
//     iter.map(move |(r, g, b)| {
//         (
//             r.saturating_mul(4),
//             g.saturating_mul(4),
//             b.saturating_mul(4),
//         )
//     })
// }

// macro_rules! iters_to_vec2 {
//     [$iter:ident $($body:tt)*] => {
//         iters_to_vec2!(@acc($iter) $($body)*)
//     };
//     [@acc($($x:tt)*) .. $fn:ident ( $($params:tt)* ) $($body:tt)*] => {
//         iters_to_vec2!(@acc($($x)* . $fn($($params)*)) $($body)*)
//     };
//     [@acc($($x:tt)*) [. $fn:ident ( $($params:tt)* ) $($cond:tt)* ] $($body:tt)*] => {
//         if $($cond)* {
//             iters_to_vec2!(@acc($fn($($x)*, $($params)*)) $($body)*)
//         } else {
//             iters_to_vec2!(@acc($($x)*) $($body)*)
//         }
//     };
//     [@acc($($x:tt)*) . $fn:ident ( $($params:tt)* ) $($body:tt)*] => {
//         iters_to_vec2!(@acc($fn($($x)*, $($params)*)) $($body)*)
//     };
//     [@acc($($x:tt)*)] => {
//         $($x)* . collect::<Vec<_>>()
//     };
// }

// #[wasm_bindgen]
// pub fn load_image3(input: Vec<u8>) -> Image {
//     let color_space = data::XYZ2SRGB;
//     let gamma = data::GAMMA_LINEAR;
//     let decoded_image = decode::new_image_from_buffer(input).unwrap();
//     let cc = ColorConversion::new(&decoded_image, color_space, gamma);

//     let image = decoded_image.image;
//     let width = decoded_image.width;
//     let height = decoded_image.height;

//     let iter = image.iter().copied();
//     let data = iters_to_vec2! (
//         iter
//             ..enumerate()
//             .pixel_info(width, height)
//             .demosaic(&image, width)
//             .black_level(512)
//             .white_scale()
//             .color_fix(&cc)
//             ..flatten()
//             .to8bit()
//     );

//     Image {
//         data,
//         width,
//         height,
//     }
// }
