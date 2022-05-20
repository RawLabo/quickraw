use super::{*, utility::*};
use std::cmp;

// http://www.brucelindbloom.com/index.html?Eqn_RGB_to_XYZ.html
pub const XYZ2ADOBE_RGB: [f32; 9] = [
    1.8037626,
    -0.49918914,
    -0.3045735,
    -1.0221082,
    1.9782866,
    0.04382154,
    0.014769779,
    -0.13003181,
    1.115262,
];

pub const XYZ2SRGB: [f32; 9] = [
    2.689655,
    -1.275862,
    -0.4137931,
    -1.0221082,
    1.9782866,
    0.04382154,
    0.061224457,
    -0.22448978,
    1.1632653,
];

pub const XYZ2RAW: [f32; 9] = [1.0, 0., 0., 0., 1.0, 0., 0., 0., 1.0];

pub const GAMMA_LINEAR: [f32; 2] = [1.0, 0.0];
pub const GAMMA_SRGB: [f32; 2] = [0.45, 4.5];

impl ColorConversion {
    const CLIP_LIMIT_I32: i32 = 65535;
    const CLIP_RANGE : (i32, i32) = (0, Self::CLIP_LIMIT_I32);

    pub fn new(raw_job : &RawJob, color_space : [f32;9], gamma: [f32;2]) -> Self {
        let color_space = matrix3_mul(&color_space, &raw_job.cam_matrix);
        let color_space = color_space.mul(1 << BIT_SHIFT);
        let white_balance = raw_job.white_balance.mul(1 << (BIT_SHIFT - log2(raw_job.white_balance[1])));
        let gamma_lut = gen_gamma_lut(gamma);
        ColorConversion {
            white_balance,
            gamma_lut,
            color_space,
        }
    }

    #[inline(always)]
    fn limit_to_range<T: Ord>(v: T, (left, right): (T, T)) -> T {
        cmp::min(cmp::max(v, left), right)
    }

    #[inline(always)]
    fn rgb_color_shift_i32((r, g, b): (i32, i32, i32), wb: &[i32; 3], c: &[i32; 9]) -> (usize, usize, usize) {
        let r = cmp::min((r * wb[0]) >> BIT_SHIFT, Self::CLIP_LIMIT_I32);
        let g = cmp::min((g * wb[1]) >> BIT_SHIFT, Self::CLIP_LIMIT_I32);
        let b = cmp::min((b * wb[2]) >> BIT_SHIFT, Self::CLIP_LIMIT_I32);

        (
            Self::limit_to_range((c[0] * r + c[1] * g + c[2] * b) >> BIT_SHIFT, Self::CLIP_RANGE) as usize,
            Self::limit_to_range((c[3] * r + c[4] * g + c[5] * b) >> BIT_SHIFT, Self::CLIP_RANGE) as usize,
            Self::limit_to_range((c[6] * r + c[7] * g + c[8] * b) >> BIT_SHIFT, Self::CLIP_RANGE) as usize,
        )
    }

    #[inline(always)]
    pub fn convert(&self, rgb: (i32, i32, i32)) -> [u16; 3] {
        let (r, g, b) = Self::rgb_color_shift_i32(rgb, &self.white_balance, &self.color_space);
        [self.gamma_lut[r], self.gamma_lut[g], self.gamma_lut[b]]
    }
}
