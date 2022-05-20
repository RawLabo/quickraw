use super::*;

pub trait ArrayMulNum<const N: usize> {
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

pub fn log2(x: i32) -> u32 {
    for i in 1..BIT_SHIFT {
        if (x >> i) == 1 {
            return i;
        }
    }
    BIT_SHIFT
}

pub fn matrix3_mul(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
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

pub fn gen_gamma_lut(gamma: [f32; 2]) -> [u16; 65536] {
    let mut gamma_map = [0u16; 65536];
    for (i, elem) in gamma_map.iter_mut().enumerate() {
        let l = i as f32 / 65535.;
        let v = if l < 0.018 {
            gamma[1] * l
        } else {
            1.099 * l.powf(gamma[0]) - 0.099
        };
        *elem = (v * 65535.) as u16;
    }
    gamma_map
}
