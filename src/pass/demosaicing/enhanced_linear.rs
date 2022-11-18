use super::*;

#[inline(always)]
fn get_window(image: &[u16], i: usize, w: usize) -> [i32; 9] {
    if let (Some(&[v1, v2, v3]), Some(&[v4, v5, v6]), Some(&[v7, v8, v9])) = (
        image.get(i - w - 1..=i - w + 1),
        image.get(i - 1..=i + 1),
        image.get(i + w - 1..=i + w + 1),
    ) {
        [
            v1 as i32, v2 as i32, v3 as i32, v4 as i32, v5 as i32, v6 as i32, v7 as i32, v8 as i32,
            v9 as i32,
        ]
    } else {
        [0; 9]
    }
}

#[inline(always)]
fn clamp(x: i32) -> u16 {
    use std::cmp::{max, min};
    min(max(x, 0), 65535) as u16
}

const DIFF_ALPHA: i32 = 16;

#[inline(always)]
fn calc_pixel_at_rb(image: &[u16], i: usize, w: usize) -> [u16; 3] {
    let [v1, v2, v3, v4, v5, v6, v7, v8, v9] = get_window(image, i, w);
    let mut green_lst = [v2, v4, v6, v8];
    green_lst.sort();

    let g = if let [Some(&p1), Some(&p2), Some(&p3), Some(&p4)] = [
        i.checked_sub(w * 2).and_then(|i| image.get(i)),
        i.checked_sub(2).and_then(|i| image.get(i)),
        image.get(i + w * 2),
        image.get(i + 2),
    ] {
        let (n, w, s, e) = (p1 as i32, p2 as i32, p3 as i32, p4 as i32);
        let diff_p = v5 * 4 - (n + w + s + e);
        (green_lst[1] + green_lst[2]) / 2 + diff_p / DIFF_ALPHA
    } else {
        (green_lst[1] + green_lst[2]) / 2
    };

    let diff_g = g * 4 - (v2 + v4 + v6 + v8);
    let third = (v1 + v3 + v7 + v9) / 4;
    let third = third + diff_g / DIFF_ALPHA;

    [v5 as u16, clamp(g), clamp(third)]
}
#[inline(always)]
fn calc_pixel_at_g(image: &[u16], i: usize, w: usize) -> [u16; 3] {
    let [v1, v2, v3, v4, v5, v6, v7, v8, v9] = get_window(image, i, w);
    let diff_g = v5 * 4 - (v1 + v3 + v7 + v9);
    let horiz = (v4 + v6) / 2 + diff_g / DIFF_ALPHA;
    let vert = (v2 + v8) / 2 + diff_g / DIFF_ALPHA;

    [clamp(horiz), v5 as u16, clamp(vert)]
}

#[inline(always)]
pub(super) fn rggb(i: usize, v: u16, image: &[u16], w: usize, h: usize) -> [u16; 3] {
    match bayer_pixel_info(i, w, h) {
        // top left corner
        (true, _, true, _, _, _) => [v, avg(image, [i + 1, i + w]), get_pixel(image, i + w + 1)],
        // top right corner
        (true, _, _, true, _, _) => [get_pixel(image, i - 1), v, get_pixel(image, i + w)],
        // bottom left corner
        (_, true, true, _, _, _) => [get_pixel(image, i - w), v, get_pixel(image, i + 1)],
        // bottom right corner
        (_, true, _, true, _, _) => [get_pixel(image, i - w - 1), avg(image, [i - w, i - 1]), v],
        // top edge
        (true, _, _, _, true, _) => [
            v,
            avg(image, [i - 1, i + w, i + 1]),
            avg(image, [i + w - 1, i + w + 1]),
        ],
        (true, _, _, _, false, _) => [avg(image, [i - 1, i + 1]), v, get_pixel(image, i + w)],
        // bottom edge
        (_, true, _, _, true, _) => [get_pixel(image, i - w), v, avg(image, [i - 1, i + 1])],
        (_, true, _, _, false, _) => [get_pixel(image, i - w - 1), avg(image, [i - w, i - 1]), v],
        // left edge
        (_, _, true, _, _, true) => [
            v,
            avg(image, [i - w, i + 1, i + w]),
            avg(image, [i - w + 1, i + w + 1]),
        ],
        (_, _, true, _, _, false) => [avg(image, [i - w, i + w]), v, get_pixel(image, i + 1)],
        // right edge
        (_, _, _, true, _, true) => [get_pixel(image, i - 1), v, get_pixel(image, i + w)],
        (_, _, _, true, _, false) => [
            avg(image, [i - w - 1, i + w - 1]),
            avg(image, [i - w, i + w, i - 1]),
            v,
        ],
        // red
        (_, _, _, _, true, true) => {
            calc_pixel_at_rb(image, i, w)
        }
        // green1
        (_, _, _, _, false, true) => {
            calc_pixel_at_g(image, i, w)
        }
        // green2
        (_, _, _, _, true, false) => {
            let [horiz, g, vert] = calc_pixel_at_g(image, i, w);
            [vert, g, horiz]
        }
        // blue
        (_, _, _, _, false, false) => {
            let [b, g, r] = calc_pixel_at_rb(image, i, w);
            [r, g, b]
        }
    }
}

#[inline(always)]
pub(super) fn bggr(i: usize, v: u16, image: &[u16], w: usize, h: usize) -> [u16; 3] {
    match bayer_pixel_info(i, w, h) {
        // top left corner
        (true, _, true, _, _, _) => [get_pixel(image, i + w + 1), avg(image, [i + 1, i + w]), v],
        // top right corner
        (true, _, _, true, _, _) => [get_pixel(image, i + w), v, get_pixel(image, i - 1)],
        // bottom left corner
        (_, true, true, _, _, _) => [get_pixel(image, i + 1), v, get_pixel(image, i - w)],
        // bottom right corner
        (_, true, _, true, _, _) => [v, avg(image, [i - w, i - 1]), get_pixel(image, i - w - 1)],
        // top edge
        (true, _, _, _, true, _) => [
            avg(image, [i + w - 1, i + w + 1]),
            avg(image, [i - 1, i + w, i + 1]),
            v,
        ],
        (true, _, _, _, false, _) => [get_pixel(image, i + w), v, avg(image, [i - 1, i + 1])],
        // bottom edge
        (_, true, _, _, true, _) => [avg(image, [i - 1, i + 1]), v, get_pixel(image, i - w)],
        (_, true, _, _, false, _) => [v, avg(image, [i - w, i - 1]), get_pixel(image, i - w - 1)],
        // left edge
        (_, _, true, _, _, true) => [
            avg(image, [i - w + 1, i + w + 1]),
            avg(image, [i - w, i + 1, i + w]),
            v,
        ],
        (_, _, true, _, _, false) => [get_pixel(image, i + 1), v, avg(image, [i - w, i + w])],
        // right edge
        (_, _, _, true, _, true) => [get_pixel(image, i + w), v, get_pixel(image, i - 1)],
        (_, _, _, true, _, false) => [
            v,
            avg(image, [i - w, i + w, i - 1]),
            avg(image, [i - w - 1, i + w - 1]),
        ],
        // blue
        (_, _, _, _, true, true) => [
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            avg(image, [i - w, i + w, i - 1, i + 1]),
            v,
        ],
        // green2
        (_, _, _, _, false, true) => [avg(image, [i - w, i + w]), v, avg(image, [i - 1, i + 1])],
        // green1
        (_, _, _, _, true, false) => [avg(image, [i - 1, i + 1]), v, avg(image, [i - w, i + w])],
        // red
        (_, _, _, _, false, false) => [
            v,
            avg(image, [i - w, i + w, i - 1, i + 1]),
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
        ],
    }
}

#[inline(always)]
pub(super) fn grbg(i: usize, v: u16, image: &[u16], w: usize, h: usize) -> [u16; 3] {
    match bayer_pixel_info(i, w, h) {
        // top left corner
        (true, _, true, _, _, _) => [get_pixel(image, i + 1), v, get_pixel(image, i + w)],
        // top right corner
        (true, _, _, true, _, _) => [v, avg(image, [i - 1, i + w]), get_pixel(image, i + w - 1)],
        // bottom left corner
        (_, true, true, _, _, _) => [get_pixel(image, i - w + 1), avg(image, [i - w, i + 1]), v],
        // bottom right corner
        (_, true, _, true, _, _) => [get_pixel(image, i - w), v, get_pixel(image, i - 1)],
        // top edge
        (true, _, _, _, true, _) => [avg(image, [i - 1, i + 1]), v, get_pixel(image, i + w)],
        (true, _, _, _, false, _) => [
            v,
            avg(image, [i - 1, i + 1, i + w]),
            avg(image, [i + w - 1, i + w + 1]),
        ],
        // bottom edge
        (_, true, _, _, true, _) => [
            avg(image, [i - w - 1, i - w + 1]),
            avg(image, [i - 1, i + 1, i - w]),
            v,
        ],
        (_, true, _, _, false, _) => [get_pixel(image, i - w), v, avg(image, [i - 1, i + 1])],
        // left edge
        (_, _, true, _, _, true) => [get_pixel(image, i + 1), v, avg(image, [i - w, i + w])],
        (_, _, true, _, _, false) => [
            avg(image, [i - w + 1, i + w + 1]),
            avg(image, [i - w, i + w, i + 1]),
            v,
        ],
        // right edge
        (_, _, _, true, _, true) => [
            v,
            avg(image, [i - w, i - 1, i + w]),
            avg(image, [i - w - 1, i + w - 1]),
        ],
        (_, _, _, true, _, false) => [avg(image, [i - w, i + w]), v, get_pixel(image, i - 1)],
        // green1
        (_, _, _, _, true, true) => [avg(image, [i - 1, i + 1]), v, avg(image, [i - w, i + w])],
        // red
        (_, _, _, _, false, true) => [
            v,
            avg(image, [i - w, i + w, i - 1, i + 1]),
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
        ],
        // blue
        (_, _, _, _, true, false) => [
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            avg(image, [i - w, i + w, i - 1, i + 1]),
            v,
        ],
        // green2
        (_, _, _, _, false, false) => [avg(image, [i - w, i + w]), v, avg(image, [i - 1, i + 1])],
    }
}

#[inline(always)]
pub(super) fn gbrg(i: usize, v: u16, image: &[u16], w: usize, h: usize) -> [u16; 3] {
    match bayer_pixel_info(i, w, h) {
        // top left corner
        (true, _, true, _, _, _) => [get_pixel(image, i + w), v, get_pixel(image, i + 1)],
        // top right corner
        (true, _, _, true, _, _) => [get_pixel(image, i + w - 1), avg(image, [i - 1, i + w]), v],
        // bottom left corner
        (_, true, true, _, _, _) => [v, avg(image, [i - w, i + 1]), get_pixel(image, i - w + 1)],
        // bottom right corner
        (_, true, _, true, _, _) => [get_pixel(image, i - 1), v, get_pixel(image, i - w)],
        // top edge
        (true, _, _, _, true, _) => [get_pixel(image, i + w), v, avg(image, [i - 1, i + 1])],
        (true, _, _, _, false, _) => [
            avg(image, [i + w - 1, i + w + 1]),
            avg(image, [i - 1, i + 1, i + w]),
            v,
        ],
        // bottom edge
        (_, true, _, _, true, _) => [
            v,
            avg(image, [i - 1, i + 1, i - w]),
            avg(image, [i - w - 1, i - w + 1]),
        ],
        (_, true, _, _, false, _) => [avg(image, [i - 1, i + 1]), v, get_pixel(image, i - w)],
        // left edge
        (_, _, true, _, _, true) => [avg(image, [i - w, i + w]), v, get_pixel(image, i + 1)],
        (_, _, true, _, _, false) => [
            v,
            avg(image, [i - w, i + w, i + 1]),
            avg(image, [i - w + 1, i + w + 1]),
        ],
        // right edge
        (_, _, _, true, _, true) => [
            avg(image, [i - w - 1, i + w - 1]),
            avg(image, [i - w, i - 1, i + w]),
            v,
        ],
        (_, _, _, true, _, false) => [get_pixel(image, i - 1), v, avg(image, [i - w, i + w])],
        // green2
        (_, _, _, _, true, true) => [avg(image, [i - w, i + w]), v, avg(image, [i - 1, i + 1])],
        // blue
        (_, _, _, _, false, true) => [
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            avg(image, [i - w, i + w, i - 1, i + 1]),
            v,
        ],
        // red
        (_, _, _, _, true, false) => [
            v,
            avg(image, [i - w, i + w, i - 1, i + 1]),
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
        ],
        // green1
        (_, _, _, _, false, false) => [avg(image, [i - 1, i + 1]), v, avg(image, [i - w, i + w])],
    }
}

