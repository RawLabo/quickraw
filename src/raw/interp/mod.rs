use super::PixelInfo;

mod utility;
pub mod none;
pub mod linear;
pub mod super_pixel;

pub trait Interp {
    fn rggb(_ : PixelInfo, image : &[u16], width: usize) -> (i32, i32, i32);
    fn grbg(_ : PixelInfo, image : &[u16], width: usize) -> (i32, i32, i32);
    fn gbrg(_ : PixelInfo, image : &[u16], width: usize) -> (i32, i32, i32);
    fn bggr(_ : PixelInfo, image : &[u16], width: usize) -> (i32, i32, i32);
}

pub struct None;
pub struct Linear;
pub struct SuperPixel;