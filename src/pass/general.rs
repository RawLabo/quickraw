use crate::raw::PixelInfo;

#[inline(always)]
pub fn pixel_info(
    iter: impl Iterator<Item = (usize, u16)>,
    width: usize,
    height: usize,
) -> impl Iterator<Item = PixelInfo> {
    iter.map(move |(i, v)| PixelInfo::new(i, v, width, height))
}

#[inline(always)]
pub fn u16rgb_to_i32rgb(iter: impl Iterator<Item = [u16; 3]>) -> impl Iterator<Item = [i32; 3]> {
    iter.map(|[r, g, b]| [r as i32, g as i32, b as i32])
}

#[inline(always)]
pub fn u16_to_u8(iter: impl Iterator<Item = u16>) -> impl Iterator<Item = u8> {
    iter.map(|v| (v / 256) as u8)
}

#[inline(always)]
pub fn sub_black_level(
    iter: impl Iterator<Item = (i32, i32, i32)>,
    black_level: i32,
) -> impl Iterator<Item = (i32, i32, i32)> {
    iter.map(move |(r, g, b)| {
        (
            r.saturating_sub(black_level),
            g.saturating_sub(black_level),
            b.saturating_sub(black_level),
        )
    })
}

#[inline(always)]
pub fn level_scale_up(
    iter: impl Iterator<Item = (i32, i32, i32)>,
    factor: i32,
) -> impl Iterator<Item = (i32, i32, i32)> {
    iter.map(move |(r, g, b)| {
        (
            r.saturating_mul(factor),
            g.saturating_mul(factor),
            b.saturating_mul(factor),
        )
    })
}
