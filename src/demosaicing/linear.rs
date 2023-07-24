use super::*;

pub(crate) fn rggb(i: usize, w: usize, h: usize, v: u16, image: &[u16]) -> [u16; 3] {
    match get_pixel_type(i, w, h) {
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
        _ => [v, v, v]
    }
}