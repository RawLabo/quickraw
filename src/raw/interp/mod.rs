use super::PixelInfo;

mod utility;
pub(in super::super) mod none;
pub(in super::super) mod linear;
pub(in super::super) mod super_pixel;

pub(in super::super) trait Interp {
    fn rggb(_ : PixelInfo, image : &[u16], width: usize) -> (i32, i32, i32);
    fn grbg(_ : PixelInfo, image : &[u16], width: usize) -> (i32, i32, i32);
    fn gbrg(_ : PixelInfo, image : &[u16], width: usize) -> (i32, i32, i32);
    fn bggr(_ : PixelInfo, image : &[u16], width: usize) -> (i32, i32, i32);
}

pub(in super::super) struct None;
pub(in super::super) struct Linear;
pub(in super::super) struct SuperPixel;