use crate::{decode::CFAPattern, utility::ArrayMulNum};

use super::*;
use pass::*;

pub struct Options<'a> {
    gamma: f32,
    color_space: &'a [f32; 9],
    no_demosaicing: bool,
}
impl<'a> Options<'a> {
    pub fn new(gamma: f32, color_space: &'a [f32; 9], no_demosaicing: bool) -> Self {
        Options {
            gamma,
            color_space,
            no_demosaicing,
        }
    }
}

pub fn load_image_from_file(
    path: &str,
    options: Options,
) -> Result<(Vec<u16>, usize, usize), RawFileReadingError> {
    let buffer = decode::get_buffer_from_file(path)?;
    load_image_from_buffer(buffer, options)
}

pub fn load_image_from_buffer(
    buffer: Vec<u8>,
    options: Options,
) -> Result<(Vec<u16>, usize, usize), RawFileReadingError> {
    let decoded_image = decode::decode_buffer(buffer)?;

    let color_matrix = utility::matrix3_mul(options.color_space, &decoded_image.cam_matrix);
    let color_matrix = color_matrix.mul(1 << BIT_SHIFT);

    let white_balance = decoded_image
        .white_balance
        .mul(1 << (BIT_SHIFT - utility::log2(decoded_image.white_balance[1])));

    let gamma_lut = gen_gamma_lut(options.gamma);

    let image = decoded_image.image;
    let width = decoded_image.width;
    let height = decoded_image.height;

    if image.len() == width * height * 3 {
        return Ok((image, width, height));
    }

    let iter = image.iter().copied();
    let data = pass::iters_to_vec! (
        iter
            ..enumerate()
            [(options.no_demosaicing, decoded_image.cfa_pattern)] {
                (true, _) => .none(),
                (false, CFAPattern::RGGB) => .linear_rggb(&image, width, height),
                (false, CFAPattern::GRBG) => .linear_grbg(&image, width, height),
                (false, CFAPattern::GBRG) => .linear_gbrg(&image, width, height),
                (false, CFAPattern::BGGR) => .linear_bggr(&image, width, height),
                (false, CFAPattern::XTrans0) => .linear_xtrans0(&image, width, height),
                (false, CFAPattern::XTrans1) => .linear_xtrans1(&image, width, height)
            }
            .gamma_correct(&gamma_lut)
            .u16rgb_to_i32rgb()
            .white_balance_fix(&white_balance)
            .color_convert(&color_matrix)
            ..flatten()
    );

    Ok((data, width, height))
}
