pub(crate) mod linear;

#[inline(always)]
fn get_pixel_type(i: usize, w: usize, h: usize) -> [bool; 6] {
    let x = i % w;
    let y = i / w;
    let is_top = y == 0;
    let is_bottom = y == h - 1;
    let is_left = x == 0;
    let is_right = x == w - 1;
    let is_column_even = x % 2 == 0;
    let is_row_even = y % 2 == 0;

    [is_top, is_bottom, is_left, is_right, is_column_even, is_row_even]
}

trait FastGet {
    fn fast_get(&self, i: usize) -> u16;
}
impl FastGet for &[u16] {
    fn fast_get(&self, i: usize) -> u16 {
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
