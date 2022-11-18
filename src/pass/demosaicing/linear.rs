use super::*;

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
        (_, _, _, _, true, true) => [
            v,
            avg(image, [i - w, i + w, i - 1, i + 1]),
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
        ],
        // green1
        (_, _, _, _, false, true) => [avg(image, [i - 1, i + 1]), v, avg(image, [i - w, i + w])],
        // green2
        (_, _, _, _, true, false) => [avg(image, [i - w, i + w]), v, avg(image, [i - 1, i + 1])],
        // blue
        (_, _, _, _, false, false) => [
            avg(image, [i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            avg(image, [i - w, i + w, i - 1, i + 1]),
            v,
        ],
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

#[inline(always)]
pub(super) fn xtrans0(i: usize, v: u16, image: &[u16], w: usize, h: usize) -> [u16; 3] {
    let x = i % w;
    let y = i / w;
    let is_top = y == 0;
    let is_left = x == 0;
    let is_bottom = y == h - 1;
    let is_right = x == w - 1;

    let index = (x % 6, y % 6);
    macro_rules! avg {
        (137) => {
            avg(image, [i - w, i - 1, i + w])
        };
        (056) => {
            avg(image, [i - w - 1, i + w - 1, i + 1])
        };
        (238) => {
            avg(image, [i + w + 1, i - 1, i + w + 1])
        };
        (157) => {
            avg(image, [i - w, i + 1, i + w])
        };
        (17) => {
            avg(image, [i - w, i + w])
        };
        (35) => {
            avg(image, [i - 1, i + 1])
        };
        (16) => {
            avg(image, [i - w, i + w - 1])
        };
        (56) => {
            avg(image, [i + 1, i + w - 1])
        };
        (23) => {
            avg(image, [i - w + 1, i - 1])
        };
        (05) => {
            avg(image, [i - w - 1, i + 1])
        };
        (07) => {
            avg(image, [i - w - 1, i + w])
        };
        (27) => {
            avg(image, [i - w + 1, i + w])
        };
        (18) => {
            avg(image, [i - w, i + w + 1])
        };
        (38) => {
            avg(image, [i - 1, i + w + 1])
        };
        (135) => {
            avg(image, [i - w, i - 1, i + 1])
        };
        (027) => {
            avg(image, [i - w - 1, i - w + 1, i + w])
        };
        (168) => {
            avg(image, [i - w, i + w - 1, i + w + 1])
        };
        (357) => {
            avg(image, [i - 1, i + 1, i + w])
        };
        (35) => {
            avg(image, [i - 1, i + 1])
        };
        (57) => {
            avg(image, [i + 1, i + w])
        };
        (37) => {
            avg(image, [i - 1, i + w])
        };
        (17) => {
            avg(image, [i - w, i + w])
        };
    }
    macro_rules! p {
        ($i:expr) => {
            get_pixel(image, $i)
        };
    }

    match (is_top, is_left, is_bottom, is_right, index) {
        (false, false, false, false, (0, 0)) => [v, avg!(137), avg!(056)],
        (false, false, false, false, (1, 0)) => [avg!(238), avg!(157), v],
        (false, false, false, false, (2, 0)) => [avg!(17), v, avg!(35)],
        (false, false, false, false, (3, 0)) => [avg!(056), avg!(137), v],
        (false, false, false, false, (4, 0)) => [v, avg!(157), avg!(238)],
        (false, false, false, false, (5, 0)) => [avg!(35), v, avg!(17)],

        (false, false, false, false, (0, 1)) => [avg!(16), v, avg!(23)],
        (false, false, false, false, (1, 1)) => [avg!(05), v, avg!(18)],
        (false, false, false, false, (2, 1)) => [v, avg!(135), avg!(027)],
        (false, false, false, false, (3, 1)) => [avg!(23), v, avg!(16)],
        (false, false, false, false, (4, 1)) => [avg!(18), v, avg!(05)],
        (false, false, false, false, (5, 1)) => [avg!(027), avg!(135), v],

        (false, false, false, false, (0, 2)) => [avg!(38), v, avg!(07)],
        (false, false, false, false, (1, 2)) => [avg!(27), v, avg!(56)],
        (false, false, false, false, (2, 2)) => [avg!(168), avg!(357), v],
        (false, false, false, false, (3, 2)) => [avg!(07), v, avg!(38)],
        (false, false, false, false, (4, 2)) => [avg!(56), v, avg!(27)],
        (false, false, false, false, (5, 2)) => [v, avg!(357), avg!(168)],

        (false, false, false, false, (0, 3)) => [avg!(056), avg!(137), v],
        (false, false, false, false, (1, 3)) => [v, avg!(157), avg!(238)],
        (false, false, false, false, (2, 3)) => [avg!(35), v, avg!(17)],
        (false, false, false, false, (3, 3)) => [v, avg!(137), avg!(056)],
        (false, false, false, false, (4, 3)) => [avg!(238), avg!(157), v],
        (false, false, false, false, (5, 3)) => [avg!(17), v, avg!(35)],

        (false, false, false, false, (0, 4)) => [avg!(23), v, avg!(16)],
        (false, false, false, false, (1, 4)) => [avg!(18), v, avg!(05)],
        (false, false, false, false, (2, 4)) => [avg!(027), avg!(135), v],
        (false, false, false, false, (3, 4)) => [avg!(16), v, avg!(23)],
        (false, false, false, false, (4, 4)) => [avg!(05), v, avg!(18)],
        (false, false, false, false, (5, 4)) => [v, avg!(135), avg!(027)],

        (false, false, false, false, (0, 5)) => [avg!(07), v, avg!(38)],
        (false, false, false, false, (1, 5)) => [avg!(56), v, avg!(27)],
        (false, false, false, false, (2, 5)) => [v, avg!(357), avg!(168)],
        (false, false, false, false, (3, 5)) => [avg!(38), v, avg!(07)],
        (false, false, false, false, (4, 5)) => [avg!(27), v, avg!(56)],
        (false, false, false, false, (5, 5)) => [avg!(168), avg!(357), v],

        (true, _, _, _, (0, _)) => [v, p!(i + w), p!(i + 1)],
        (true, _, _, _, (1, _)) => [p!(i - 1), avg!(57), v],
        (true, _, _, _, (2, _)) => [p!(i + w), v, avg!(35)],
        (true, _, _, _, (3, _)) => [p!(i + 1), avg!(37), v],
        (true, _, _, _, (4, _)) => [v, p!(i + w), p!(i - 1)],
        (true, _, _, _, (5, _)) => [p!(i - 1), v, p!(i + w)],

        (_, _, true, _, (0, _)) => [p!(i - w + 2), v, p!(i + 2)],
        (_, _, true, _, (1, _)) => [p!(i + 1), v, p!(i - w + 1)],
        (_, _, true, _, (2, _)) => [v, avg!(35), p!(i - w)],
        (_, _, true, _, (3, _)) => [p!(i - 1), v, p!(i - w - 1)],
        (_, _, true, _, (4, _)) => [p!(i - w + 1), v, p!(i + 1)],
        (_, _, true, _, (5, _)) => [p!(i - w), p!(i - 1), v],

        (_, true, _, _, (_, 0)) => [v, p!(i + w), p!(i + 1)],
        (_, true, _, _, (_, 1)) => [p!(i - w), v, p!(i - w + 1)],
        (_, true, _, _, (_, 2)) => [p!(i + w + 1), v, p!(i + w)],
        (_, true, _, _, (_, 3)) => [p!(i + 1), avg!(17), v],
        (_, true, _, _, (_, 4)) => [p!(i - w + 1), v, p!(i - w)],
        (_, true, _, _, (_, 5)) => [p!(i + 2), v, p!(i - w * 2)],

        (_, _, _, true, (_, 1)) => [p!(i + w), p!(i - 1), v],
        (_, _, _, true, (_, 2)) => [v, p!(i - 1), p!(i - w)],
        (_, _, _, true, (_, 3)) => [avg!(17), v, p!(i - 1)],
        (_, _, _, true, (_, 4)) => [v, p!(i - 1), p!(i + w)],
        (_, _, _, true, (_, 5)) => [p!(i - w), p!(i - 1), v],

        _ => [0; 3],
    }
}

#[inline(always)]
pub(super) fn xtrans1(i: usize, v: u16, image: &[u16], w: usize, h: usize) -> [u16; 3] {
    let x = i % w;
    let y = i / w;
    let is_top = y == 0;
    let is_left = x == 0;
    let is_bottom = y == h - 1;
    let is_right = x == w - 1;

    let index = (x % 6, y % 6);
    macro_rules! avg {
        (137) => {
            avg(image, [i - w, i - 1, i + w])
        };
        (056) => {
            avg(image, [i - w - 1, i + w - 1, i + 1])
        };
        (238) => {
            avg(image, [i + w + 1, i - 1, i + w + 1])
        };
        (157) => {
            avg(image, [i - w, i + 1, i + w])
        };
        (17) => {
            avg(image, [i - w, i + w])
        };
        (35) => {
            avg(image, [i - 1, i + 1])
        };
        (16) => {
            avg(image, [i - w, i + w - 1])
        };
        (56) => {
            avg(image, [i + 1, i + w - 1])
        };
        (23) => {
            avg(image, [i - w + 1, i - 1])
        };
        (05) => {
            avg(image, [i - w - 1, i + 1])
        };
        (07) => {
            avg(image, [i - w - 1, i + w])
        };
        (27) => {
            avg(image, [i - w + 1, i + w])
        };
        (18) => {
            avg(image, [i - w, i + w + 1])
        };
        (38) => {
            avg(image, [i - 1, i + w + 1])
        };
        (135) => {
            avg(image, [i - w, i - 1, i + 1])
        };
        (027) => {
            avg(image, [i - w - 1, i - w + 1, i + w])
        };
        (168) => {
            avg(image, [i - w, i + w - 1, i + w + 1])
        };
        (357) => {
            avg(image, [i - 1, i + 1, i + w])
        };
        (35) => {
            avg(image, [i - 1, i + 1])
        };
        (57) => {
            avg(image, [i + 1, i + w])
        };
        (37) => {
            avg(image, [i - 1, i + w])
        };
        (17) => {
            avg(image, [i - w, i + w])
        };
    }
    macro_rules! p {
        ($i:expr) => {
            get_pixel(image, $i)
        };
    }

    match (is_top, is_left, is_bottom, is_right, index) {
        (false, false, false, false, (0, 5)) => [v, avg!(137), avg!(056)],
        (false, false, false, false, (1, 5)) => [avg!(238), avg!(157), v],
        (false, false, false, false, (2, 5)) => [avg!(17), v, avg!(35)],
        (false, false, false, false, (3, 5)) => [avg!(056), avg!(137), v],
        (false, false, false, false, (4, 5)) => [v, avg!(157), avg!(238)],
        (false, false, false, false, (5, 5)) => [avg!(35), v, avg!(17)],

        (false, false, false, false, (0, 0)) => [avg!(16), v, avg!(23)],
        (false, false, false, false, (1, 0)) => [avg!(05), v, avg!(18)],
        (false, false, false, false, (2, 0)) => [v, avg!(135), avg!(027)],
        (false, false, false, false, (3, 0)) => [avg!(23), v, avg!(16)],
        (false, false, false, false, (4, 0)) => [avg!(18), v, avg!(05)],
        (false, false, false, false, (5, 0)) => [avg!(027), avg!(135), v],

        (false, false, false, false, (0, 1)) => [avg!(38), v, avg!(07)],
        (false, false, false, false, (1, 1)) => [avg!(27), v, avg!(56)],
        (false, false, false, false, (2, 1)) => [avg!(168), avg!(357), v],
        (false, false, false, false, (3, 1)) => [avg!(07), v, avg!(38)],
        (false, false, false, false, (4, 1)) => [avg!(56), v, avg!(27)],
        (false, false, false, false, (5, 1)) => [v, avg!(357), avg!(168)],

        (false, false, false, false, (0, 2)) => [avg!(056), avg!(137), v],
        (false, false, false, false, (1, 2)) => [v, avg!(157), avg!(238)],
        (false, false, false, false, (2, 2)) => [avg!(35), v, avg!(17)],
        (false, false, false, false, (3, 2)) => [v, avg!(137), avg!(056)],
        (false, false, false, false, (4, 2)) => [avg!(238), avg!(157), v],
        (false, false, false, false, (5, 2)) => [avg!(17), v, avg!(35)],

        (false, false, false, false, (0, 3)) => [avg!(23), v, avg!(16)],
        (false, false, false, false, (1, 3)) => [avg!(18), v, avg!(05)],
        (false, false, false, false, (2, 3)) => [avg!(027), avg!(135), v],
        (false, false, false, false, (3, 3)) => [avg!(16), v, avg!(23)],
        (false, false, false, false, (4, 3)) => [avg!(05), v, avg!(18)],
        (false, false, false, false, (5, 3)) => [v, avg!(135), avg!(027)],

        (false, false, false, false, (0, 4)) => [avg!(07), v, avg!(38)],
        (false, false, false, false, (1, 4)) => [avg!(56), v, avg!(27)],
        (false, false, false, false, (2, 4)) => [v, avg!(357), avg!(168)],
        (false, false, false, false, (3, 4)) => [avg!(38), v, avg!(07)],
        (false, false, false, false, (4, 4)) => [avg!(27), v, avg!(56)],
        (false, false, false, false, (5, 4)) => [avg!(168), avg!(357), v],

        (true, _, _, _, (0, _)) => [p!(i + 2), v, p!(i + w + 2)],
        (true, _, _, _, (1, _)) => [p!(i + 1), v, p!(i + w + 1)],
        (true, _, _, _, (2, _)) => [v, avg!(35), p!(i + w)],
        (true, _, _, _, (3, _)) => [p!(i - 1), v, p!(i + w - 1)],
        (true, _, _, _, (4, _)) => [p!(i + w + 1), v, p!(i + 1)],
        (true, _, _, _, (5, _)) => [p!(i + w), p!(i - 1), v],

        (_, _, true, _, (0, _)) => [v, p!(i - w), p!(i + 1)],
        (_, _, true, _, (1, _)) => [p!(i - 1), p!(i - w), v],
        (_, _, true, _, (2, _)) => [p!(i - w), v, avg!(35)],
        (_, _, true, _, (3, _)) => [p!(i + 1), p!(i - w), v],
        (_, _, true, _, (4, _)) => [v, p!(i - w), p!(i - 1)],
        (_, _, true, _, (5, _)) => [p!(i - 1), v, p!(i - w)],

        (_, true, _, _, (_, 0)) => [p!(i + 2), v, p!(i + w + 2)],
        (_, true, _, _, (_, 1)) => [p!(i + w + 1), v, p!(i + w)],
        (_, true, _, _, (_, 2)) => [p!(i + 1), avg!(17), v],
        (_, true, _, _, (_, 3)) => [p!(i - w + 1), v, p!(i - w)],
        (_, true, _, _, (_, 4)) => [p!(i + 2), v, p!(i - w + 2)],
        (_, true, _, _, (_, 5)) => [v, p!(i - w), p!(i + 1)],

        (_, _, _, true, (_, 1)) => [v, p!(i - 1), p!(i - w)],
        (_, _, _, true, (_, 2)) => [avg!(17), v, p!(i - 1)],
        (_, _, _, true, (_, 3)) => [v, p!(i - 1), p!(i + w)],
        (_, _, _, true, (_, 4)) => [p!(i - w), p!(i - 1), v],
        (_, _, _, true, (_, 5)) => [p!(i - 1), v, p!(i - w)],

        _ => [0; 3],
    }
}
