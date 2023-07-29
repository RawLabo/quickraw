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
pub(crate) fn rggb(i: usize, w: usize, stat: &[bool; 6], image: &[u16]) -> [u16; 3] {
    let v = image.fast_get(i);

    match stat {
        // center
        [false, false, false, false, true, true] => {
            let (a, b) = avg_corner_4(image, i, w);
            [v, b, a]
        }
        [false, false, false, false, false, true] => {
            let (a, b) = avg_tb_lr(image, i, w);
            [b, v, a]
        }
        [false, false, false, false, true, false] => {
            let (a, b) = avg_tb_lr(image, i, w);
            [a, v, b]
        }
        [false, false, false, false, false, false] => {
            let (a, b) = avg_corner_4(image, i, w);
            [a, b, v]
        }
        // top left | top even
        [true, _, true, _, _, _] | [true, _, _, _, true, _] => {
            [v, image.fast_get(i + 1), image.fast_get(i + w + 1)]
        }
        // top right | top odd
        [true, _, _, true, _, _] | [true, _, _, _, false, _] => {
            [image.fast_get(i - 1), v, image.fast_get(i + w)]
        }
        // bottom left | bottom even
        [_, true, true, _, _, _] | [_, true, _, _, true, _] => {
            [image.fast_get(i - w), v, image.fast_get(i + 1)]
        }
        // bottom right | bottom odd
        [_, true, _, true, _, _] | [_, true, _, _, false, _] => {
            [image.fast_get(i - w - 1), image.fast_get(i - 1), v]
        }
        // left even
        [_, _, true, _, _, true] => [v, image.fast_get(i + 1), image.fast_get(i + w + 1)],
        // left odd
        [_, _, true, _, _, false] => [image.fast_get(i + w), v, image.fast_get(i + 1)],
        // right even
        [_, _, _, true, _, true] => [image.fast_get(i - 1), v, image.fast_get(i + w)],
        // right odd
        [_, _, _, true, _, false] => [image.fast_get(i + w - 1), image.fast_get(i - 1), v],
    }
}
