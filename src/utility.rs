use super::*;
use once_cell::sync::Lazy;

pub(super) trait ArrayMulNum<const N: usize> {
    fn mul(&self, factor: i32) -> [i32; N];
}
macro_rules! gen_array_mul_num_impls {
    ($t:ty) => {
        impl<const N: usize> ArrayMulNum<N> for [$t; N] {
            fn mul(&self, factor: i32) -> [i32; N] {
                let factor = factor as $t;
                let mut result: [i32; N] = [0i32; N];
                for (i, &v) in self.iter().enumerate() {
                    result[i] = (factor * v) as i32;
                }
                result
            }
        }
    };
}
gen_array_mul_num_impls!(f32);
gen_array_mul_num_impls!(i32);

pub(super) fn log2(x: i32) -> u32 {
    for i in 1..BIT_SHIFT {
        if (x >> i) == 1 {
            return i;
        }
    }
    BIT_SHIFT
}

pub(super) fn matrix3_mul(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    [
        a[0] * b[0] + a[1] * b[3] + a[2] * b[6],
        a[0] * b[1] + a[1] * b[4] + a[2] * b[7],
        a[0] * b[2] + a[1] * b[5] + a[2] * b[8],
        a[3] * b[0] + a[4] * b[3] + a[5] * b[6],
        a[3] * b[1] + a[4] * b[4] + a[5] * b[7],
        a[3] * b[2] + a[4] * b[5] + a[5] * b[8],
        a[6] * b[0] + a[7] * b[3] + a[8] * b[6],
        a[6] * b[1] + a[7] * b[4] + a[8] * b[7],
        a[6] * b[2] + a[7] * b[5] + a[8] * b[8],
    ]
}

pub(super) fn gen_gamma_lut(gamma: [f32; 2]) -> [u16; 65536] {
    let mut gamma_map = [0u16; 65536];
    for (i, elem) in gamma_map.iter_mut().enumerate() {
        let l = i as f32 / 65535.;
        *elem = (l.powf(gamma[0]) * 65535.) as u16;
    }
    gamma_map
}


pub(super) static BASIC_INFO_RULE : Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    quickexif::describe_rule!(tiff {
        0x010f {
            str + 0 / make
        }
        0x0110 {
            str + 0 / model
        }
        0xc612? / dng_version
        if dng_version ? {
            0xc614 {
                str + 0 / make_model
            }
            0xc622 {
                r64 + 0 / c0
                r64 + 1 / c1
                r64 + 2 / c2
                r64 + 3 / c3
                r64 + 4 / c4
                r64 + 5 / c5
                r64 + 6 / c6
                r64 + 7 / c7
                r64 + 8 / c8
            }
        }
    })
});