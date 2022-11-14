
#[inline(always)]
pub fn u16rgb_to_i32rgb(iter: impl Iterator<Item = [u16; 3]>) -> impl Iterator<Item = [i32; 3]> {
    iter.map(|[r, g, b]| [r as i32, g as i32, b as i32])
}

// #[inline(always)]
// pub fn sub_black_level(
//     iter: impl Iterator<Item = u16>,
//     black_level: u16,
// ) -> impl Iterator<Item = u16> {
//     iter.map(move |v| v.saturating_sub(black_level))
// }

// #[inline(always)]
// pub fn level_scale_up(
//     iter: impl Iterator<Item = u16>,
//     factor: u16,
// ) -> impl Iterator<Item = u16> {
//     iter.map(move |v| v.saturating_mul(factor))
// }
