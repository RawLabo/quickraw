pub(crate) mod linear;

enum PixelType {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopOdd,
    TopEven,
    LeftOdd,
    LeftEven,
    RightOdd,
    RightEven,
    BottomOdd,
    BottomEven,
    Center0,
    Center1,
    Center2,
    Center3,
}

fn get_pixel_type(i: usize, w: usize, h: usize) -> PixelType {
    let x = i % w;
    let y = i / w;
    let is_top = y == 0;
    let is_left = x == 0;
    let is_bottom = y == h - 1;
    let is_right = x == w - 1;
    let is_column_even = x % 2 == 0;
    let is_row_even = y % 2 == 0;

    match (
        is_top,
        is_bottom,
        is_left,
        is_right,
        is_column_even,
        is_row_even,
    ) {
        // top left corner
        (true, _, true, _, _, _) => PixelType::TopLeft,
        // top right corner
        (true, _, _, true, _, _) => PixelType::TopRight,
        // bottom left corner
        (_, true, true, _, _, _) => PixelType::BottomLeft,
        // bottom right corner
        (_, true, _, true, _, _) => PixelType::BottomRight,
        // top edge
        (true, _, _, _, true, _) => PixelType::TopEven,
        (true, _, _, _, false, _) => PixelType::TopOdd,
        // bottom edge
        (_, true, _, _, true, _) => PixelType::BottomEven,
        (_, true, _, _, false, _) => PixelType::BottomOdd,
        // left edge
        (_, _, true, _, _, true) => PixelType::LeftEven,
        (_, _, true, _, _, false) => PixelType::LeftOdd,
        // right edge
        (_, _, _, true, _, true) => PixelType::RightEven,
        (_, _, _, true, _, false) => PixelType::RightOdd,
        // center
        (_, _, _, _, true, true) => PixelType::Center0,
        (_, _, _, _, false, true) => PixelType::Center1,
        (_, _, _, _, true, false) => PixelType::Center2,
        (_, _, _, _, false, false) => PixelType::Center3,
    }
}


#[inline(always)]
fn avg_tb_lr(image: &[u16], i: usize, w: usize) -> (u16, u16) {
    let a = image[i - w] as u32;
    let b = image[i + w] as u32;
    let c = image[i - 1] as u32;
    let d = image[i + 1] as u32;

    let x = (a + b) / 2;
    let y = (c + d) / 2;
    (x as u16, y as u16)
}

#[inline(always)]
fn avg_corner_4(image: &[u16], i: usize, w: usize) -> (u16, u16) {
    let top: usize = i - w;
    let bottom: usize = i + w;

    let a = image[top - 1] as u32;
    let b = image[top + 1] as u32;
    let c = image[bottom - 1] as u32;
    let d = image[bottom + 1] as u32;

    let e = image[top] as u32;
    let f = image[bottom] as u32;
    let g = image[i - 1] as u32;
    let h = image[i + 1] as u32;

    let x = (a + b + c + d) / 4;
    let y = (e + f + g + h) / 4;
    (x as u16, y as u16)
}
