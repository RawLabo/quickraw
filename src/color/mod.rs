use crate::parse::WhiteBalance;

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
    // pub(crate) fn fix_simd(&self, [r, g, b]: [u16; 3]) -> [u32; 3] {
    //     use wide::u32x4;
    //     let a = u32x4::from([r as u32, g as u32, b as u32, 0]);
    //     let b = u32x4::from([self.r, self.g, self.b, 0]);
    //     let c: u32x4 = a * b >> self.bit_shift;
    //     let d = c.min(u32x4::from([0xffff, 0xffff, 0xffff, 0]));
    //     let result = d.as_array_ref();
    //     [result[0], result[1], result[2]]
    // }
    pub(crate) fn fix(&self, [r, g, b]: [u16; 3]) -> [u32; 3] {
        [
            std::cmp::min(r as u32 * self.r >> self.bit_shift, 0xffff),
            std::cmp::min(g as u32 * self.g >> self.bit_shift, 0xffff),
            std::cmp::min(b as u32 * self.b >> self.bit_shift, 0xffff),
        ]
    }
}
