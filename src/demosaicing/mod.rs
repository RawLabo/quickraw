pub(crate) mod linear;

pub(crate) enum PixelType {
    TopLeft,
    TopEven,
    TopOdd,
    TopRight,
    LeftEven,
    LeftOdd,
    RightEven,
    RightOdd,
    BottomLeft,
    BottomEven,
    BottomOdd,
    BottomRight,
    Center0,
    Center1,
    Center2,
    Center3,
}
pub(crate) struct PixelInfo {
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    is_column_even: bool,
    is_row_even: bool,
}
impl PixelInfo {
    pub(crate) fn new(w: usize, h: usize) -> PixelInfo {
        Self {
            w,
            h,
            x: 0,
            y: 0,
            is_column_even: true,
            is_row_even: true,
        }
    }

    #[inline(always)]
    pub(crate) fn get_stat_and_update(&mut self) -> PixelType {
        let is_top = self.y == 0;
        let is_bottom = self.y == self.h - 1;
        let is_left = self.x == 0;
        let is_right = self.x == self.w - 1;

        let ret = match [
            is_top,
            is_bottom,
            is_left,
            is_right,
            self.is_column_even,
            self.is_row_even,
        ] {
            [false, false, false, false, true, true] => PixelType::Center0,
            [false, false, false, false, false, true] => PixelType::Center1,
            [false, false, false, false, true, false] => PixelType::Center2,
            [false, false, false, false, false, false] => PixelType::Center3,
            [true, _, true, _, _, _] => PixelType::TopLeft,
            [true, _, _, _, true, _] => PixelType::TopEven,
            [true, _, _, true, _, _] => PixelType::TopRight,
            [true, _, _, _, false, _] => PixelType::TopOdd,
            [_, true, true, _, _, _] => PixelType::BottomLeft,
            [_, true, _, _, true, _] => PixelType::BottomEven,
            [_, true, _, true, _, _] => PixelType::BottomRight,
            [_, true, _, _, false, _] => PixelType::BottomOdd,
            [_, _, true, _, _, true] => PixelType::LeftEven,
            [_, _, true, _, _, false] => PixelType::LeftOdd,
            [_, _, _, true, _, true] => PixelType::RightEven,
            [_, _, _, true, _, false] => PixelType::RightOdd,
        };

        // update for next pixel
        if is_right {
            self.x = 0;
            self.y += 1;
            self.is_row_even = !self.is_row_even;
        } else {
            self.x += 1;
        }
        self.is_column_even = !self.is_column_even;

        ret
    }
}

trait FastGet {
    fn fast_get(&self, i: usize) -> u16;
}
impl FastGet for &[u16] {
    fn fast_get(&self, i: usize) -> u16 {
        // SAFETY: the index has been checked before the usage in this mod
        unsafe { *self.get_unchecked(i) }
    }
}

#[inline(always)]
fn avg_tb_lr(image: &[u16], i: usize, w: usize) -> (u16, u16) {
    let a = image.fast_get(i - w) as u32;
    let b = image.fast_get(i + w) as u32;
    let c = image.fast_get(i - 1) as u32;
    let d = image.fast_get(i + 1) as u32;

    let x = (a + b) / 2;
    let y = (c + d) / 2;
    (x as u16, y as u16)
}

#[inline(always)]
fn avg_corner_4(image: &[u16], i: usize, w: usize) -> (u16, u16) {
    let top: usize = i - w;
    let bottom: usize = i + w;

    let a = image.fast_get(top - 1) as u32;
    let b = image.fast_get(top + 1) as u32;
    let c = image.fast_get(bottom - 1) as u32;
    let d = image.fast_get(bottom + 1) as u32;

    let e = image.fast_get(top) as u32;
    let f = image.fast_get(bottom) as u32;
    let g = image.fast_get(i - 1) as u32;
    let h = image.fast_get(i + 1) as u32;

    let x = (a + b + c + d) / 4;
    let y = (e + f + g + h) / 4;
    (x as u16, y as u16)
}

pub(crate) trait Demosaicing {
    fn demosaicing(i: usize, w: usize, stat: PixelType, image: &[u16]) -> [u16; 3];
}