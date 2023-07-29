use super::*;

pub(crate) fn grbg(_i: usize, _w: usize, _h: usize, _image: &[u16]) -> [u16; 3] {
    todo!()
}
pub(crate) fn gbrg(_i: usize, _w: usize, _h: usize, _image: &[u16]) -> [u16; 3] {
    todo!()
}
pub(crate) fn bggr(_i: usize, _w: usize, _h: usize, _image: &[u16]) -> [u16; 3] {
    todo!()
}
pub(crate) fn xtrans0(_i: usize, _w: usize, _h: usize, _image: &[u16]) -> [u16; 3] {
    todo!()
}
pub(crate) fn xtrans1(_i: usize, _w: usize, _h: usize, _image: &[u16]) -> [u16; 3] {
    todo!()
}

#[inline(always)]
pub(crate) fn rggb(i: usize, w: usize, stat: PixelType, image: &[u16]) -> [u16; 3] {
    let v = image.fast_get(i);

    match stat {
        // center
        PixelType::Center0 => {
            let (a, b) = avg_corner_4(image, i, w);
            [v, b, a]
        }
        PixelType::Center1 => {
            let (a, b) = avg_tb_lr(image, i, w);
            [b, v, a]
        }
        PixelType::Center2 => {
            let (a, b) = avg_tb_lr(image, i, w);
            [a, v, b]
        }
        PixelType::Center3 => {
            let (a, b) = avg_corner_4(image, i, w);
            [a, b, v]
        }
        // top left | top even
        PixelType::TopLeft | PixelType::TopEven => {
            [v, image.fast_get(i + 1), image.fast_get(i + w + 1)]
        }
        // top right | top odd
        PixelType::TopRight | PixelType::TopOdd => {
            [image.fast_get(i - 1), v, image.fast_get(i + w)]
        }
        // bottom left | bottom even
        PixelType::BottomLeft | PixelType::BottomEven => {
            [image.fast_get(i - w), v, image.fast_get(i + 1)]
        }
        // bottom right | bottom odd
        PixelType::BottomRight | PixelType::BottomOdd => {
            [image.fast_get(i - w - 1), image.fast_get(i - 1), v]
        }
        // left even
        PixelType::LeftEven => [v, image.fast_get(i + 1), image.fast_get(i + w + 1)],
        // left odd
        PixelType::LeftOdd => [image.fast_get(i + w), v, image.fast_get(i + 1)],
        // right even
        PixelType::RightEven => [image.fast_get(i - 1), v, image.fast_get(i + w)],
        // right odd
        PixelType::RightOdd => [image.fast_get(i + w - 1), image.fast_get(i - 1), v],
    }
}
