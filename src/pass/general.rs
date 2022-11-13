#[derive(Debug)]
pub struct PixelInfo {
    pub i: usize,
    pub v: u16,
    pub x: usize,
    pub y: usize,
    pub is_top: bool,
    pub is_left: bool,
    pub is_bottom: bool,
    pub is_right: bool,
    pub is_column_even: bool,
    pub is_row_even: bool,
}

impl PixelInfo {
    #[inline(always)]
    pub(crate) fn new(i: usize, v: u16, w: usize, h: usize) -> Self {
        let x = i % w;
        let y = i / w;
        let is_top = y == 0;
        let is_left = x == 0;
        let is_bottom = y == h - 1;
        let is_right = x == w - 1;
        let is_column_even = x % 2 == 0;
        let is_row_even = y % 2 == 0;

        Self {
            i,
            v,
            x,
            y,
            is_top,
            is_bottom,
            is_left,
            is_right,
            is_column_even,
            is_row_even,
        }
    }
}

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
