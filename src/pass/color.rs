use crate::ColorConversion;
use std::cmp;

const BIT_SHIFT: u32 = 13u32;
const CLIP_LIMIT_I32: i32 = 65535;
const CLIP_RANGE: (i32, i32) = (0, CLIP_LIMIT_I32);

#[inline(always)]
pub fn white_balance_fix<'a>(
    iter: impl Iterator<Item = [i32; 3]> + 'a,
    white_balance: &'a [i32; 3],
) -> impl Iterator<Item = [i32; 3]> + 'a {
    iter.map(move |[r, g, b]| {
        let r = cmp::min((r * white_balance[0]) >> BIT_SHIFT, CLIP_LIMIT_I32);
        let g = cmp::min((g * white_balance[1]) >> BIT_SHIFT, CLIP_LIMIT_I32);
        let b = cmp::min((b * white_balance[2]) >> BIT_SHIFT, CLIP_LIMIT_I32);
        [r, g, b]
    })
}

// #[inline(always)]
// pub fn white_balance_fix<'a>(
//     iter: impl Iterator<Item = (i32, i32, i32)> + 'a,
//     white_balance: &'a [i32; 3],
// ) -> impl Iterator<Item = (i32, i32, i32)> + 'a {
//     iter.map(move |(r, g, b)| {
//         let rgbx4 = i32x4::from([r, g, b, 0]);
//         let wbx4 = i32x4::from([white_balance[0], white_balance[1], white_balance[2], 0]);
//         let [r, g, b, _]: [i32; 4] = (rgbx4 * wbx4).into();

//         let r = cmp::min(r >> BIT_SHIFT, CLIP_LIMIT_I32);
//         let g = cmp::min(g >> BIT_SHIFT, CLIP_LIMIT_I32);
//         let b = cmp::min(b >> BIT_SHIFT, CLIP_LIMIT_I32);
//         (r, g, b)
//     })
// }

#[inline(always)]
pub fn color_convert<'a>(
    iter: impl Iterator<Item = [i32; 3]> + 'a,
    c: &'a [i32; 9],
) -> impl Iterator<Item = [u16; 3]> + 'a {
    iter.map(move |[r, g, b]| {
        [
            limit_to_range((c[0] * r + c[1] * g + c[2] * b) >> BIT_SHIFT, CLIP_RANGE) as u16,
            limit_to_range((c[3] * r + c[4] * g + c[5] * b) >> BIT_SHIFT, CLIP_RANGE) as u16,
            limit_to_range((c[6] * r + c[7] * g + c[8] * b) >> BIT_SHIFT, CLIP_RANGE) as u16,
        ]
    })
}

// #[inline(always)]
// pub fn color_convert<'a>(
//     iter: impl Iterator<Item = (i32, i32, i32)> + 'a,
//     c: &'a [i32; 9],
// ) -> impl Iterator<Item = [u16; 3]> + 'a {
//     iter.map(move |(r, g, b)| {
//         let p1 = i32x8::from([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
//         let p2 = i32x8::from([r, g, b, r, g, b, r, g]);
//         let p: [i32; 8] = (p1 * p2).into();

//         [
//             limit_to_range((p[0] + p[1] + p[2]) >> BIT_SHIFT, CLIP_RANGE) as u16,
//             limit_to_range((p[3] + p[4] + p[5]) >> BIT_SHIFT, CLIP_RANGE) as u16,
//             limit_to_range((p[6] + p[7] + c[8] * b) >> BIT_SHIFT, CLIP_RANGE) as u16,
//         ]
//     })
// }

#[inline(always)]
pub fn gamma_correct<'a>(
    iter: impl Iterator<Item = [u16; 3]> + 'a,
    gamma_lut: &'a [u16; 65536],
) -> impl Iterator<Item = [u16; 3]> + 'a {
    iter.map(|[r, g, b]| {
        [
            gamma_lut[r as usize],
            gamma_lut[g as usize],
            gamma_lut[b as usize],
        ]
    })
}

pub fn gen_gamma_lut(gamma: f32) -> [u16; 65536] {
    let mut lut = [0u16; 65536];
    for (i, elem) in lut.iter_mut().enumerate() {
        let l = i as f32 / 65535.;
        *elem = (l.powf(gamma) * 65535.) as u16;
    }
    lut
}

#[inline(always)]
fn limit_to_range<T: Ord>(v: T, (left, right): (T, T)) -> T {
    cmp::min(cmp::max(v, left), right)
}
