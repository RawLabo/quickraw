use crate::raw::PixelInfo;

#[inline(always)]
pub fn demosaic<'a>(
    iter: impl Iterator<Item = PixelInfo> + 'a,
    image: &'a [u16],
    width: usize,
) -> impl Iterator<Item = (i32, i32, i32)> + 'a {
    iter.map(move |pi| rggb(pi, image, width))
}

#[inline(always)]
fn get_pixel(image: &[u16], i: usize) -> i32 {
    unsafe { *image.get_unchecked(i) as i32 }
}
#[inline(always)]
fn avg<const N: usize>(image: &[u16], indexes: [usize; N]) -> i32 {
    indexes.iter().map(|&i| get_pixel(image, i)).sum::<i32>() / N as i32
}

#[inline(always)]
fn rggb(
    PixelInfo {
        i,
        v,
        x: _,
        y: _,
        is_top,
        is_left,
        is_bottom,
        is_right,
        is_column_even,
        is_row_even,
    }: PixelInfo,
    image: &[u16],
    w: usize,
) -> (i32, i32, i32) {
    match (
        is_top,
        is_bottom,
        is_left,
        is_right,
        is_column_even,
        is_row_even,
    ) {
        // top left corner
        (true, _, true, _, _, _) => (v, avg(image, [i + 1, i + w]), get_pixel(image, i + w + 1)),
        // top right corner
        (true, _, _, true, _, _) => (get_pixel(image, i - 1), v, get_pixel(image, i + w)),
        // bottom left corner
        (_, true, true, _, _, _) => (get_pixel(image, i - w), v, get_pixel(image, i + 1)),
        // bottom right corner
        (_, true, _, true, _, _) => (get_pixel(image, i - w - 1), avg(image, [i - w, i - 1]), v),
        // top edge
        (true, _, _, _, true, _) => (
            v,
            avg(image, [i - 1, i + w, i + 1]),
            avg(image, [i + w - 1, i + w + 1]),
        ),
        (true, _, _, _, false, _) => (avg(image, [i - 1, i + 1]), v, get_pixel(image, i + w)),
        // bottom edge
        (_, true, _, _, true, _) => (get_pixel(image, i - w), v, avg(image, [i - 1, i + 1])),
        (_, true, _, _, false, _) => (get_pixel(image, i - w - 1), avg(image, [i - w, i - 1]), v),
        // left edge
        (_, _, true, _, _, true) => (
            v,
            avg(image, [i - w, i + 1, i + w]),
            avg(image, [i - w + 1, i + w + 1]),
        ),
        (_, _, true, _, _, false) => (avg(image, [i - w, i + w]), v, get_pixel(image, i + 1)),
        // right edge
        (_, _, _, true, _, true) => (get_pixel(image, i - 1), v, get_pixel(image, i + w)),
        (_, _, _, true, _, false) => (
            avg(image, [i - w - 1, i + w - 1]),
            avg(image, [i - w, i + w, i - 1]),
            v,
        ),
        // red
        (_, _, _, _, true, true) => (
            v,
            avg(image, [i - w, i + w, i - 1, i + 1]),
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
        ),
        // green1
        (_, _, _, _, false, true) => (avg(image, [i - 1, i + 1]), v, avg(image, [i - w, i + w])),
        // green2
        (_, _, _, _, true, false) => (avg(image, [i - w, i + w]), v, avg(image, [i - 1, i + 1])),
        // blue
        (_, _, _, _, false, false) => (
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            avg(image, [i - w, i + w, i - 1, i + 1]),
            v,
        ),
    }
}
