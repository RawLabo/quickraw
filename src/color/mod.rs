use crate::parse::{ColorMatrix, WhiteBalance};
use wide::i32x4;

pub(crate) fn gen_gamma_lut(gamma: f32) -> [u16; 65536] {
    let mut lut = [0u16; 65536];
    for (i, elem) in lut.iter_mut().enumerate() {
        let l = i as f32 / 65535.;
        *elem = (l.powf(gamma) * 65535.) as u16;
    }
    lut
}

#[inline(always)]
pub(crate) fn gamma_correct([r, g, b]: [usize; 3], gamma_lut: &[u16; 65536]) -> [u16; 3] {
    [gamma_lut[r], gamma_lut[g], gamma_lut[b]]
}

impl WhiteBalance {
    #[inline(always)]
    pub(crate) fn fix(&self, [r, g, b]: [u16; 3]) -> [i32; 3] {
        let rgb = i32x4::from([r as i32, g as i32, b as i32, 0]);
        let r1 = rgb * self.rgb >> self.bit_shift;

        #[cfg(target_feature="avx")] // x64 can benefit from the direct construction of i32x4 value for clamping
        let r1 = r1.min(i32x4::splat(0xffff));
        #[cfg(not(target_feature="avx"))]
        let r1 = r1.min(self.clamp);

        let r2 = r1.as_array_ref();
        [r2[0], r2[1], r2[2]]
    }
}

impl ColorMatrix {
    const BIT_SCALE: i32 = 14;
    const COLOR_MATRIX_SCALE: f32 = 16384f32;

    #[inline(always)]
    pub(crate) fn shift_color(&self, [r, g, b]: [i32; 3]) -> [usize; 3] {
        let r = i32x4::splat(r);
        let g = i32x4::splat(g);
        let b = i32x4::splat(b);

        let r = (r * self.column0 + g * self.column1 + b * self.column2) >> Self::BIT_SCALE;

        #[cfg(target_feature="avx")]
        let r1 = r.min(i32x4::splat(0xffff)).max(i32x4::splat(0));
        #[cfg(not(target_feature="avx"))]
        let r1 = r.min(self.clamp0).max(self.clamp1);

        let r2 = r1.as_array_ref();
        [r2[0] as usize, r2[1] as usize, r2[2] as usize]
    }

    /// self.matrix_with_colorspace = color_space * self.matrix *
    pub(crate) fn update_colorspace(&mut self, color_space: &[f32; 9]) {
        let a = color_space;
        let b = self.matrix;

        let [c0, c1, c2, c3, c4, c5, c6, c7, c8] = [
            ((a[0] * b[0] + a[1] * b[3] + a[2] * b[6]) * Self::COLOR_MATRIX_SCALE) as i32,
            ((a[0] * b[1] + a[1] * b[4] + a[2] * b[7]) * Self::COLOR_MATRIX_SCALE) as i32,
            ((a[0] * b[2] + a[1] * b[5] + a[2] * b[8]) * Self::COLOR_MATRIX_SCALE) as i32,
            ((a[3] * b[0] + a[4] * b[3] + a[5] * b[6]) * Self::COLOR_MATRIX_SCALE) as i32,
            ((a[3] * b[1] + a[4] * b[4] + a[5] * b[7]) * Self::COLOR_MATRIX_SCALE) as i32,
            ((a[3] * b[2] + a[4] * b[5] + a[5] * b[8]) * Self::COLOR_MATRIX_SCALE) as i32,
            ((a[6] * b[0] + a[7] * b[3] + a[8] * b[6]) * Self::COLOR_MATRIX_SCALE) as i32,
            ((a[6] * b[1] + a[7] * b[4] + a[8] * b[7]) * Self::COLOR_MATRIX_SCALE) as i32,
            ((a[6] * b[2] + a[7] * b[5] + a[8] * b[8]) * Self::COLOR_MATRIX_SCALE) as i32,
        ];

        self.column0 = i32x4::from([c0, c3, c6, 0]);
        self.column1 = i32x4::from([c1, c4, c7, 0]);
        self.column2 = i32x4::from([c2, c5, c8, 0]);
    }
}
