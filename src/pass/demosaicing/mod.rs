use super::PixelInfo;

mod linear;

#[inline(always)]
pub fn none<'a>(
    iter: impl Iterator<Item = PixelInfo> + 'a,
) -> impl Iterator<Item = [u16; 3]> + 'a {
    iter.map(|pi| [pi.v, pi.v, pi.v])
}

macro_rules! gen_linear {
    ($name:ident, $fn:ident) => {
        #[inline(always)]
        pub fn $name<'a>(
            iter: impl Iterator<Item = PixelInfo> + 'a,
            image: &'a [u16],
            width: usize,
        ) -> impl Iterator<Item = [u16; 3]> + 'a {
            iter.map(move |pi| linear::$fn(pi, image, width))
        }
    };
}

gen_linear!(linear_rggb, rggb);
gen_linear!(linear_bggr, bggr);
gen_linear!(linear_grbg, grbg);
gen_linear!(linear_gbrg, gbrg);
gen_linear!(linear_xtrans0, xtrans0);
gen_linear!(linear_xtrans1, xtrans1);


#[inline(always)]
pub(self) fn get_pixel(image: &[u16], i: usize) -> u16 {
    unsafe { *image.get_unchecked(i) }
}
#[inline(always)]
pub(self) fn avg<const N: usize>(image: &[u16], indexes: [usize; N]) -> u16 {
    (indexes
        .iter()
        .map(|&i| get_pixel(image, i) as u32)
        .sum::<u32>()
        / N as u32) as u16
}
