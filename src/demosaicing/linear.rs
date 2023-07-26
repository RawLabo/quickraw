use super::*;

#[inline(always)]
pub(crate) fn rggb(i: usize, w: usize, h: usize, v: u16, image: &[u16]) -> [u16; 3] {
    match get_pixel_type(i, w, h) {
        PixelType::TopLeft | PixelType::TopEven => [v, image[i + 1], image[i + w + 1]],
        PixelType::TopRight | PixelType::TopOdd => [image[i - 1], v, image[i + w]],
        PixelType::BottomLeft | PixelType::BottomEven => [image[i - w], v, image[i + 1]],
        PixelType::BottomRight | PixelType::BottomOdd => [image[i - w - 1], image[i - 1], v],
        PixelType::LeftEven => [v, image[i + 1], image[i + w + 1]],
        PixelType::LeftOdd => [image[i + w], v, image[i + 1]],
        PixelType::RightEven => [image[i - 1], v, image[i + w]],
        PixelType::RightOdd => [image[i + w - 1], image[i - 1], v],
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
    }
}
