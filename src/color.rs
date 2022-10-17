use raw::DecodedImage;

use super::{*, utility::*};
use std::cmp;

impl ColorConversion {
    const CLIP_LIMIT_I32: i32 = 65535;
    const CLIP_RANGE : (i32, i32) = (0, Self::CLIP_LIMIT_I32);

    pub(super) fn new(decoded_image : &DecodedImage, color_space : [f32;9], gamma: [f32;2]) -> Self {
        let color_space = matrix3_mul(&color_space, &decoded_image.cam_matrix);
        let color_space = color_space.mul(1 << BIT_SHIFT);
        let white_balance = decoded_image.white_balance.mul(1 << (BIT_SHIFT - log2(decoded_image.white_balance[1])));
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
    pub(super) fn convert(&self, rgb: (i32, i32, i32)) -> [u16; 3] {
        let (r, g, b) = Self::rgb_color_shift_i32(rgb, &self.white_balance, &self.color_space);
        [self.gamma_lut[r], self.gamma_lut[g], self.gamma_lut[b]]
    }
}
