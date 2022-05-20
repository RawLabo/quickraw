use super::{utility::ImageOp, *};

impl SuperPixel {
    #[inline(always)]
    pub fn rggb(i: usize, v: u16, image: &[u16], w: usize) -> (i32, i32, i32) {
        (v as i32, image.avg([i + 1, i + w]), image.get_pixel(i + w + 1))
    }

    #[inline(always)]
    pub fn bggr(i: usize, v: u16, image: &[u16], w: usize) -> (i32, i32, i32) {
        (image.get_pixel(i + w + 1), image.avg([i + 1, i + w]), v as i32)
    }

    #[inline(always)]
    pub fn grbg(i: usize, _: u16, image: &[u16], w: usize) -> (i32, i32, i32) {
        (
            image.get_pixel(i + 1),
            image.avg([i, i + w + 1]),
            image.get_pixel(i + w),
        )
    }

    #[inline(always)]
    pub fn gbrg(i: usize, _: u16, image: &[u16], w: usize) -> (i32, i32, i32) {
        (
            image.get_pixel(i + w),
            image.avg([i, i + w + 1]),
            image.get_pixel(i + 1),
        )
    }
}
