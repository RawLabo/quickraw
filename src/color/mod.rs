pub(crate) fn gen_gamma_lut(gamma: f32) -> [u16;65536] {
    let mut lut = [0u16; 65536];
    for (i, elem) in lut.iter_mut().enumerate() {
        let l = i as f32 / 65535.;
        *elem = (l.powf(gamma) * 65535.) as u16;
    }
    lut
}

#[inline(always)]
pub(crate) fn gamma_correct([r, g, b]: [u32; 3], gamma_lut: &[u16;65536]) -> [u16; 3] {
    [
        gamma_lut[r as usize],
        gamma_lut[g as usize],
        gamma_lut[b as usize],
    ]
}
