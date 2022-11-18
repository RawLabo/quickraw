mod linear;
mod enhanced_linear;

#[inline(always)]
pub fn none<'a>(
    iter: impl Iterator<Item = (usize, u16)> + 'a,
) -> impl Iterator<Item = [u16; 3]> + 'a {
    iter.map(|(_, v)| [v; 3])
}

macro_rules! gen_linear {
    ($name:ident, $fn:expr) => {
        #[inline(always)]
        pub fn $name<'a>(
            iter: impl Iterator<Item = (usize, u16)> + 'a,
            image: &'a [u16],
            width: usize,
            height: usize,
        ) -> impl Iterator<Item = [u16; 3]> + 'a {
            iter.map(move |(i, v)| $fn(i, v, image, width, height))
        }
    };
}

gen_linear!(linear_rggb, linear::rggb);
gen_linear!(linear_bggr, linear::bggr);
gen_linear!(linear_grbg, linear::grbg);
gen_linear!(linear_gbrg, linear::gbrg);
gen_linear!(linear_xtrans0, linear::xtrans0);
gen_linear!(linear_xtrans1, linear::xtrans1);

gen_linear!(elinear_rggb, enhanced_linear::rggb);
gen_linear!(elinear_bggr, enhanced_linear::bggr);
gen_linear!(elinear_grbg, enhanced_linear::grbg);
gen_linear!(elinear_gbrg, enhanced_linear::gbrg);

#[inline(always)]
pub(self) fn get_pixel(image: &[u16], i: usize) -> u16 {
    unsafe { *image.get_unchecked(i) }
}
#[inline(always)]
pub(self) fn avg<const N: usize>(image: &[u16], indexes: [usize; N]) -> u16 {
    (indexes
        .into_iter()
        .map(|i| get_pixel(image, i) as u32)
        .sum::<u32>()
        / N as u32) as u16
}
#[inline(always)]
pub(self) fn bayer_pixel_info(i: usize, w: usize, h: usize) -> (bool, bool, bool, bool, bool, bool) {
    let x = i % w;
    let y = i / w;
    let is_top = y == 0;
    let is_left = x == 0;
    let is_bottom = y == h - 1;
    let is_right = x == w - 1;
    let is_column_even = x % 2 == 0;
    let is_row_even = y % 2 == 0;
    (
        is_top,
        is_bottom,
        is_left,
        is_right,
        is_column_even,
        is_row_even,
    )
}