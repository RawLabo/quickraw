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
    pub(crate) fn fix(&self, [r, g, b]: [u16; 3]) -> [u32; 3] {
        [
            std::cmp::min(r as u32 * self.r >> self.bit_shift, 0xffff),
            std::cmp::min(g as u32 * self.g >> self.bit_shift, 0xffff),
            std::cmp::min(b as u32 * self.b >> self.bit_shift, 0xffff),
        ]
    }
}
