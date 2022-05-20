use super::{utility::ImageOp, *};

impl Linear {
    #[inline(always)]
    pub fn xtrans0(
        PixelInfo {
            i,
            v,
            x,
            y,
            is_top,
            is_left,
            is_bottom,
            is_right,
            is_column_even: _,
            is_row_even: _,
        }: PixelInfo,
        image: &[u16],
        w: usize,
    ) -> (i32, i32, i32) {
        let v = v as i32;
        let index = (x % 6, y % 6);
        macro_rules! avg {
            (137) => {image.avg([i - w, i - 1, i + w])};
            (056) => {image.avg([i - w - 1, i + w - 1, i + 1])};
            (238) => {image.avg([i + w + 1,i - 1, i + w + 1])};
            (157) => {image.avg([i - w,i + 1, i + w])};
            (17) => {image.avg([i - w,i + w])};
            (35) => {image.avg([i - 1,i + 1])};
            (16) => {image.avg([i - w, i + w - 1])};
            (56) => {image.avg([i + 1, i + w - 1])};
            (23) => {image.avg([i - w + 1, i - 1])};
            (05) => {image.avg([i - w - 1, i + 1])};
            (07) => {image.avg([i - w - 1, i + w])};
            (27) => {image.avg([i - w + 1, i + w])};
            (18) => {image.avg([i - w, i + w + 1])};
            (38) => {image.avg([i - 1, i + w + 1])};
            (135) => {image.avg([i - w, i - 1, i + 1])};
            (027) => {image.avg([i - w - 1, i - w + 1, i + w])};
            (168) => {image.avg([i - w, i + w - 1, i + w + 1])};
            (357) => {image.avg([i - 1, i + 1, i + w])};
            (35) => {image.avg([i - 1, i + 1])};
            (57) => {image.avg([i + 1, i + w])};
            (37) => {image.avg([i - 1, i + w])};
            (17) => {image.avg([i - w, i + w])};
        }
        macro_rules! p {
            ($i:expr) => { image.get_pixel($i) }
        }

        match (is_top, is_left, is_bottom, is_right, index) {
            (false, false, false, false, (0, 0)) => (v, avg!(137), avg!(056)),
            (false, false, false, false, (1, 0)) => (avg!(238), avg!(157), v),
            (false, false, false, false, (2, 0)) => (avg!(17), v, avg!(35)),
            (false, false, false, false, (3, 0)) => (avg!(056), avg!(137), v),
            (false, false, false, false, (4, 0)) => (v, avg!(157), avg!(238)),
            (false, false, false, false, (5, 0)) => (avg!(35), v, avg!(17)),

            (false, false, false, false, (0, 1)) => (avg!(16), v, avg!(23)),
            (false, false, false, false, (1, 1)) => (avg!(05), v, avg!(18)),
            (false, false, false, false, (2, 1)) => (v, avg!(135), avg!(027)),
            (false, false, false, false, (3, 1)) => (avg!(23), v, avg!(16)),
            (false, false, false, false, (4, 1)) => (avg!(18), v, avg!(05)),
            (false, false, false, false, (5, 1)) => (avg!(027), avg!(135), v),

            (false, false, false, false, (0, 2)) => (avg!(38), v, avg!(07)),
            (false, false, false, false, (1, 2)) => (avg!(27), v, avg!(56)),
            (false, false, false, false, (2, 2)) => (avg!(168), avg!(357), v),
            (false, false, false, false, (3, 2)) => (avg!(07), v, avg!(38)),
            (false, false, false, false, (4, 2)) => (avg!(56), v, avg!(27)),
            (false, false, false, false, (5, 2)) => (v, avg!(357), avg!(168)),

            (false, false, false, false, (0, 3)) => (avg!(056), avg!(137), v),
            (false, false, false, false, (1, 3)) => (v, avg!(157), avg!(238)),
            (false, false, false, false, (2, 3)) => (avg!(35), v, avg!(17)),
            (false, false, false, false, (3, 3)) => (v, avg!(137), avg!(056)),
            (false, false, false, false, (4, 3)) => (avg!(238), avg!(157), v),
            (false, false, false, false, (5, 3)) => (avg!(17), v, avg!(35)),

            (false, false, false, false, (0, 4)) => (avg!(23), v, avg!(16)),
            (false, false, false, false, (1, 4)) => (avg!(18), v, avg!(05)),
            (false, false, false, false, (2, 4)) => (avg!(027), avg!(135), v),
            (false, false, false, false, (3, 4)) => (avg!(16), v, avg!(23)),
            (false, false, false, false, (4, 4)) => (avg!(05), v, avg!(18)),
            (false, false, false, false, (5, 4)) => (v, avg!(135), avg!(027)),

            (false, false, false, false, (0, 5)) => (avg!(07), v, avg!(38)),
            (false, false, false, false, (1, 5)) => (avg!(56), v, avg!(27)),
            (false, false, false, false, (2, 5)) => (v, avg!(357), avg!(168)),
            (false, false, false, false, (3, 5)) => (avg!(38), v, avg!(07)),
            (false, false, false, false, (4, 5)) => (avg!(27), v, avg!(56)),
            (false, false, false, false, (5, 5)) => (avg!(168), avg!(357), v),

            (true, _, _, _ , (0, _)) => (v, p!(i + w), p!(i + 1)),
            (true, _, _, _ , (1, _)) => (p!(i - 1), avg!(57), v),
            (true, _, _, _ , (2, _)) => (p!(i + w), v, avg!(35)),
            (true, _, _, _ , (3, _)) => (p!(i + 1), avg!(37), v),
            (true, _, _, _ , (4, _)) => (v, p!(i + w), p!(i - 1)),
            (true, _, _, _ , (5, _)) => (p!(i - 1), v, p!(i + w)),

            (_, _, true, _ , (0, _)) => (p!(i - w + 2), v, p!(i + 2)),
            (_, _, true, _ , (1, _)) => (p!(i + 1), v, p!(i - w + 1)),
            (_, _, true, _ , (2, _)) => (v, avg!(35), p!(i - w)),
            (_, _, true, _ , (3, _)) => (p!(i - 1), v, p!(i - w - 1)),
            (_, _, true, _ , (4, _)) => (p!(i - w + 1), v, p!(i + 1)),
            (_, _, true, _ , (5, _)) => (p!(i - w), p!(i - 1), v),

            (_, true, _, _, (_, 0)) => (v, p!(i + w), p!(i + 1)),
            (_, true, _, _, (_, 1)) => (p!(i - w), v, p!(i - w + 1)),
            (_, true, _, _, (_, 2)) => (p!(i + w + 1), v, p!(i + w)),
            (_, true, _, _, (_, 3)) => (p!(i + 1), avg!(17), v),
            (_, true, _, _, (_, 4)) => (p!(i - w + 1), v, p!(i - w)),
            (_, true, _, _, (_, 5)) => (p!(i + 2), v, p!(i - w * 2)),

            (_, _, _, true, (_, 1)) => (p!(i + w), p!(i - 1), v),
            (_, _, _, true, (_, 2)) => (v, p!(i - 1), p!(i - w)),
            (_, _, _, true, (_, 3)) => (avg!(17), v, p!(i - 1)),
            (_, _, _, true, (_, 4)) => (v, p!(i - 1), p!(i + w)),
            (_, _, _, true, (_, 5)) => (p!(i - w), p!(i - 1), v),

            _ => (0, 0, 0)
        }
    }

    #[inline(always)]
    pub fn xtrans1(
        PixelInfo {
            i,
            v,
            x,
            y,
            is_top,
            is_left,
            is_bottom,
            is_right,
            is_column_even: _,
            is_row_even: _,
        }: PixelInfo,
        image: &[u16],
        w: usize,
    ) -> (i32, i32, i32) {
        let v = v as i32;
        let index = (x % 6, y % 6);
        macro_rules! avg {
            (137) => {image.avg([i - w, i - 1, i + w])};
            (056) => {image.avg([i - w - 1, i + w - 1, i + 1])};
            (238) => {image.avg([i + w + 1,i - 1, i + w + 1])};
            (157) => {image.avg([i - w,i + 1, i + w])};
            (17) => {image.avg([i - w,i + w])};
            (35) => {image.avg([i - 1,i + 1])};
            (16) => {image.avg([i - w, i + w - 1])};
            (56) => {image.avg([i + 1, i + w - 1])};
            (23) => {image.avg([i - w + 1, i - 1])};
            (05) => {image.avg([i - w - 1, i + 1])};
            (07) => {image.avg([i - w - 1, i + w])};
            (27) => {image.avg([i - w + 1, i + w])};
            (18) => {image.avg([i - w, i + w + 1])};
            (38) => {image.avg([i - 1, i + w + 1])};
            (135) => {image.avg([i - w, i - 1, i + 1])};
            (027) => {image.avg([i - w - 1, i - w + 1, i + w])};
            (168) => {image.avg([i - w, i + w - 1, i + w + 1])};
            (357) => {image.avg([i - 1, i + 1, i + w])};
            (35) => {image.avg([i - 1, i + 1])};
            (57) => {image.avg([i + 1, i + w])};
            (37) => {image.avg([i - 1, i + w])};
            (17) => {image.avg([i - w, i + w])};
        }
        macro_rules! p {
            ($i:expr) => { image.get_pixel($i) }
        }

        match (is_top, is_left, is_bottom, is_right, index) {
            (false, false, false, false, (0, 5)) => (v, avg!(137), avg!(056)),
            (false, false, false, false, (1, 5)) => (avg!(238), avg!(157), v),
            (false, false, false, false, (2, 5)) => (avg!(17), v, avg!(35)),
            (false, false, false, false, (3, 5)) => (avg!(056), avg!(137), v),
            (false, false, false, false, (4, 5)) => (v, avg!(157), avg!(238)),
            (false, false, false, false, (5, 5)) => (avg!(35), v, avg!(17)),

            (false, false, false, false, (0, 0)) => (avg!(16), v, avg!(23)),
            (false, false, false, false, (1, 0)) => (avg!(05), v, avg!(18)),
            (false, false, false, false, (2, 0)) => (v, avg!(135), avg!(027)),
            (false, false, false, false, (3, 0)) => (avg!(23), v, avg!(16)),
            (false, false, false, false, (4, 0)) => (avg!(18), v, avg!(05)),
            (false, false, false, false, (5, 0)) => (avg!(027), avg!(135), v),

            (false, false, false, false, (0, 1)) => (avg!(38), v, avg!(07)),
            (false, false, false, false, (1, 1)) => (avg!(27), v, avg!(56)),
            (false, false, false, false, (2, 1)) => (avg!(168), avg!(357), v),
            (false, false, false, false, (3, 1)) => (avg!(07), v, avg!(38)),
            (false, false, false, false, (4, 1)) => (avg!(56), v, avg!(27)),
            (false, false, false, false, (5, 1)) => (v, avg!(357), avg!(168)),

            (false, false, false, false, (0, 2)) => (avg!(056), avg!(137), v),
            (false, false, false, false, (1, 2)) => (v, avg!(157), avg!(238)),
            (false, false, false, false, (2, 2)) => (avg!(35), v, avg!(17)),
            (false, false, false, false, (3, 2)) => (v, avg!(137), avg!(056)),
            (false, false, false, false, (4, 2)) => (avg!(238), avg!(157), v),
            (false, false, false, false, (5, 2)) => (avg!(17), v, avg!(35)),

            (false, false, false, false, (0, 3)) => (avg!(23), v, avg!(16)),
            (false, false, false, false, (1, 3)) => (avg!(18), v, avg!(05)),
            (false, false, false, false, (2, 3)) => (avg!(027), avg!(135), v),
            (false, false, false, false, (3, 3)) => (avg!(16), v, avg!(23)),
            (false, false, false, false, (4, 3)) => (avg!(05), v, avg!(18)),
            (false, false, false, false, (5, 3)) => (v, avg!(135), avg!(027)),

            (false, false, false, false, (0, 4)) => (avg!(07), v, avg!(38)),
            (false, false, false, false, (1, 4)) => (avg!(56), v, avg!(27)),
            (false, false, false, false, (2, 4)) => (v, avg!(357), avg!(168)),
            (false, false, false, false, (3, 4)) => (avg!(38), v, avg!(07)),
            (false, false, false, false, (4, 4)) => (avg!(27), v, avg!(56)),
            (false, false, false, false, (5, 4)) => (avg!(168), avg!(357), v),

            (true, _, _, _ , (0, _)) => (p!(i + 2), v, p!(i + w + 2)),
            (true, _, _, _ , (1, _)) => (p!(i + 1), v, p!(i + w + 1)),
            (true, _, _, _ , (2, _)) => (v, avg!(35), p!(i + w)),
            (true, _, _, _ , (3, _)) => (p!(i - 1), v, p!(i + w - 1)),
            (true, _, _, _ , (4, _)) => (p!(i + w + 1), v, p!(i + 1)),
            (true, _, _, _ , (5, _)) => (p!(i + w), p!(i - 1), v),

            (_, _, true, _ , (0, _)) => (v, p!(i - w), p!(i + 1)),
            (_, _, true, _ , (1, _)) => (p!(i - 1), p!(i - w), v),
            (_, _, true, _ , (2, _)) => (p!(i - w), v, avg!(35)),
            (_, _, true, _ , (3, _)) => (p!(i + 1), p!(i - w), v),
            (_, _, true, _ , (4, _)) => (v, p!(i - w), p!(i - 1)),
            (_, _, true, _ , (5, _)) => (p!(i - 1), v, p!(i - w)),

            (_, true, _, _, (_, 0)) => (p!(i + 2), v, p!(i + w + 2)),
            (_, true, _, _, (_, 1)) => (p!(i + w + 1), v, p!(i + w)),
            (_, true, _, _, (_, 2)) => (p!(i + 1), avg!(17), v),
            (_, true, _, _, (_, 3)) => (p!(i - w + 1), v, p!(i - w)),
            (_, true, _, _, (_, 4)) => (p!(i + 2), v, p!(i - w + 2)),
            (_, true, _, _, (_, 5)) => (v, p!(i - w), p!(i + 1)),

            (_, _, _, true, (_, 1)) => (v, p!(i - 1), p!(i - w)),
            (_, _, _, true, (_, 2)) => (avg!(17), v, p!(i - 1)),
            (_, _, _, true, (_, 3)) => (v, p!(i - 1), p!(i + w)),
            (_, _, _, true, (_, 4)) => (p!(i - w), p!(i - 1), v),
            (_, _, _, true, (_, 5)) => (p!(i - 1), v, p!(i - w)),

            _ => (0, 0, 0)
        }
    }
}


impl Interp for Linear {
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
        match (is_top, is_bottom, is_left, is_right, is_column_even, is_row_even) {
            // top left corner
            (true, _, true, _, _, _) => (v, image.avg([i + 1, i + w]), image.get_pixel(i + w + 1)),
            // top right corner
            (true, _, _, true, _, _) => (image.get_pixel(i - 1), v, image.get_pixel(i + w)),
            // bottom left corner
            (_, true, true, _, _, _) => (image.get_pixel(i - w), v, image.get_pixel(i + 1)),
            // bottom right corner
            (_, true, _, true, _, _) => (image.get_pixel(i - w - 1), image.avg([i - w, i - 1]), v),
            // top edge
            (true, _, _, _, true, _) => (v, image.avg([i - 1, i + w, i + 1]), image.avg([i + w - 1, i + w + 1])),
            (true, _, _, _, false, _) => (image.avg([i - 1, i + 1]), v, image.get_pixel(i + w)),
            // bottom edge
            (_, true, _, _, true, _) => (image.get_pixel(i - w), v, image.avg([i - 1, i + 1])),
            (_, true, _, _, false, _) => (image.get_pixel(i - w - 1), image.avg([i - w, i - 1]), v),
            // left edge
            (_, _, true, _, _, true) => (v, image.avg([i - w, i + 1, i + w]), image.avg([i - w + 1, i + w + 1])),
            (_, _, true, _, _, false) => (image.avg([i - w, i + w]), v, image.get_pixel(i + 1)),
            // right edge
            (_, _, _, true, _, true) => (image.get_pixel(i - 1), v, image.get_pixel(i + w)),
            (_, _, _, true, _, false) => (image.avg([i - w - 1, i + w - 1]), image.avg([i - w, i + w, i - 1]), v),
            // red
            (_, _, _, _, true, true) => (
                v,
                image.avg([i - w, i + w, i - 1, i + 1]),
                image.avg([i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            ),
            // green1
            (_, _, _, _, false, true) => (image.avg([i - 1, i + 1]), v, image.avg([i - w, i + w])),
            // green2
            (_, _, _, _, true, false) => (image.avg([i - w, i + w]), v, image.avg([i - 1, i + 1])),
            // blue
            (_, _, _, _, false, false) => (
                image.avg([i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
                image.avg([i - w, i + w, i - 1, i + 1]),
                v,
            ),
        }
    }

    #[inline(always)]
    fn bggr(
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
        match (is_top, is_bottom, is_left, is_right, is_column_even, is_row_even) {
            // top left corner
            (true, _, true, _, _, _) => (image.get_pixel(i + w + 1), image.avg([i + 1, i + w]), v),
            // top right corner
            (true, _, _, true, _, _) => (image.get_pixel(i + w), v, image.get_pixel(i - 1)),
            // bottom left corner
            (_, true, true, _, _, _) => (image.get_pixel(i + 1), v, image.get_pixel(i - w)),
            // bottom right corner
            (_, true, _, true, _, _) => (v, image.avg([i - w, i - 1]), image.get_pixel(i - w - 1)),
            // top edge
            (true, _, _, _, true, _) => (image.avg([i + w - 1, i + w + 1]), image.avg([i - 1, i + w, i + 1]), v),
            (true, _, _, _, false, _) => (image.get_pixel(i + w), v, image.avg([i - 1, i + 1])),
            // bottom edge
            (_, true, _, _, true, _) => (image.avg([i - 1, i + 1]), v, image.get_pixel(i - w)),
            (_, true, _, _, false, _) => (v, image.avg([i - w, i - 1]), image.get_pixel(i - w - 1)),
            // left edge
            (_, _, true, _, _, true) => (image.avg([i - w + 1, i + w + 1]), image.avg([i - w, i + 1, i + w]), v),
            (_, _, true, _, _, false) => (image.get_pixel(i + 1), v, image.avg([i - w, i + w])),
            // right edge
            (_, _, _, true, _, true) => (image.get_pixel(i + w), v, image.get_pixel(i - 1)),
            (_, _, _, true, _, false) => (v, image.avg([i - w, i + w, i - 1]), image.avg([i - w - 1, i + w - 1])),
            // blue
            (_, _, _, _, true, true) => (
                image.avg([i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
                image.avg([i - w, i + w, i - 1, i + 1]),
                v,
            ),
            // green2
            (_, _, _, _, false, true) => (image.avg([i - w, i + w]), v, image.avg([i - 1, i + 1])),
            // green1
            (_, _, _, _, true, false) => (image.avg([i - 1, i + 1]), v, image.avg([i - w, i + w])),
            // red
            (_, _, _, _, false, false) => (
                v,
                image.avg([i - w, i + w, i - 1, i + 1]),
                image.avg([i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            ),
        }
    }

    #[inline(always)]
    fn grbg(
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
        match (is_top, is_bottom, is_left, is_right, is_column_even, is_row_even) {
            // top left corner
            (true, _, true, _, _, _) => (image.get_pixel(i + 1), v, image.get_pixel(i + w)),
            // top right corner
            (true, _, _, true, _, _) => (v, image.avg([i - 1, i + w]), image.get_pixel(i + w - 1)),
            // bottom left corner
            (_, true, true, _, _, _) => (image.get_pixel(i - w + 1), image.avg([i - w, i + 1]), v),
            // bottom right corner
            (_, true, _, true, _, _) => (image.get_pixel(i - w), v, image.get_pixel(i - 1)),
            // top edge
            (true, _, _, _, true, _) => (image.avg([i - 1, i + 1]), v, image.get_pixel(i + w)),
            (true, _, _, _, false, _) => (v, image.avg([i - 1, i + 1, i + w]), image.avg([i + w - 1, i + w + 1])),
            // bottom edge
            (_, true, _, _, true, _) => (image.avg([i - w - 1, i - w + 1]), image.avg([i - 1, i + 1, i - w]), v),
            (_, true, _, _, false, _) => (image.get_pixel(i - w), v, image.avg([i - 1, i + 1])),
            // left edge
            (_, _, true, _, _, true) => (image.get_pixel(i + 1), v, image.avg([i - w, i + w])),
            (_, _, true, _, _, false) => (image.avg([i - w + 1, i + w + 1]), image.avg([i - w, i + w, i + 1]), v),
            // right edge
            (_, _, _, true, _, true) => (v, image.avg([i - w, i - 1, i + w]), image.avg([i - w - 1, i + w - 1])),
            (_, _, _, true, _, false) => (image.avg([i - w, i + w]), v, image.get_pixel(i - 1)),
            // green1
            (_, _, _, _, true, true) => (image.avg([i - 1, i + 1]), v, image.avg([i - w, i + w])),
            // red
            (_, _, _, _, false, true) => (
                v,
                image.avg([i - w, i + w, i - 1, i + 1]),
                image.avg([i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            ),
            // blue
            (_, _, _, _, true, false) => (
                image.avg([i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
                image.avg([i - w, i + w, i - 1, i + 1]),
                v,
            ),
            // green2
            (_, _, _, _, false, false) => (image.avg([i - w, i + w]), v, image.avg([i - 1, i + 1])),
        }
    }

    #[inline(always)]
    fn gbrg(
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
        match (is_top, is_bottom, is_left, is_right, is_column_even, is_row_even) {
            // top left corner
            (true, _, true, _, _, _) => (image.get_pixel(i + w), v, image.get_pixel(i + 1)),
            // top right corner
            (true, _, _, true, _, _) => (image.get_pixel(i + w - 1), image.avg([i - 1, i + w]), v),
            // bottom left corner
            (_, true, true, _, _, _) => (v, image.avg([i - w, i + 1]), image.get_pixel(i - w + 1)),
            // bottom right corner
            (_, true, _, true, _, _) => (image.get_pixel(i - 1), v, image.get_pixel(i - w)),
            // top edge
            (true, _, _, _, true, _) => (image.get_pixel(i + w), v, image.avg([i - 1, i + 1])),
            (true, _, _, _, false, _) => (image.avg([i + w - 1, i + w + 1]), image.avg([i - 1, i + 1, i + w]), v),
            // bottom edge
            (_, true, _, _, true, _) => (v, image.avg([i - 1, i + 1, i - w]), image.avg([i - w - 1, i - w + 1])),
            (_, true, _, _, false, _) => (image.avg([i - 1, i + 1]), v, image.get_pixel(i - w)),
            // left edge
            (_, _, true, _, _, true) => (image.avg([i - w, i + w]), v, image.get_pixel(i + 1)),
            (_, _, true, _, _, false) => (v, image.avg([i - w, i + w, i + 1]), image.avg([i - w + 1, i + w + 1])),
            // right edge
            (_, _, _, true, _, true) => (image.avg([i - w - 1, i + w - 1]), image.avg([i - w, i - 1, i + w]), v),
            (_, _, _, true, _, false) => (image.get_pixel(i - 1), v, image.avg([i - w, i + w])),
            // green2
            (_, _, _, _, true, true) => (image.avg([i - w, i + w]), v, image.avg([i - 1, i + 1])),
            // blue
            (_, _, _, _, false, true) => (
                image.avg([i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
                image.avg([i - w, i + w, i - 1, i + 1]),
                v,
            ),
            // red
            (_, _, _, _, true, false) => (
                v,
                image.avg([i - w, i + w, i - 1, i + 1]),
                image.avg([i - w - 1, i - w + 1, i + w - 1, i + w + 1]),
            ),
            // green1
            (_, _, _, _, false, false) => (image.avg([i - 1, i + 1]), v, image.avg([i - w, i + w])),
        }
    }
}
