use super::*;

impl Interp for None {
    #[inline(always)]
    fn rggb(
        PixelInfo {
            i: _,
            v,
            x: _,
            y: _,
            is_top: _,
            is_left: _,
            is_bottom: _,
            is_right: _,
            is_column_even,
            is_row_even,
        }: PixelInfo,
        _: &[u16],
        _: usize,
    ) -> (i32, i32, i32) {
        match (is_column_even, is_row_even) {
            (true, true) => (v, 0, 0),
            (false, true) => (0, v, 0),
            (true, false) => (0, v, 0),
            (false, false) => (0, 0, v),
        }
    }

    #[inline(always)]
    fn bggr(
        PixelInfo {
            i: _,
            v,
            x: _,
            y: _,
            is_top: _,
            is_left: _,
            is_bottom: _,
            is_right: _,
            is_column_even,
            is_row_even,
        }: PixelInfo,
        _: &[u16],
        _: usize,
    ) -> (i32, i32, i32) {
        match (is_column_even, is_row_even) {
            (true, true) => (0, 0, v),
            (false, true) => (0, v, 0),
            (true, false) => (0, v, 0),
            (false, false) => (v, 0, 0),
        }
    }
    #[inline(always)]
    fn grbg(
        PixelInfo {
            i: _,
            v,
            x: _,
            y: _,
            is_top: _,
            is_left: _,
            is_bottom: _,
            is_right: _,
            is_column_even,
            is_row_even,
        }: PixelInfo,
        _: &[u16],
        _: usize,
    ) -> (i32, i32, i32) {
        match (is_column_even, is_row_even) {
            (true, true) => (0, v, 0),
            (false, true) => (v, 0, 0),
            (true, false) => (0, 0, v),
            (false, false) => (0, v, 0),
        }
    }

    #[inline(always)]
    fn gbrg(
        PixelInfo {
            i: _,
            v,
            x: _,
            y: _,
            is_top: _,
            is_left: _,
            is_bottom: _,
            is_right: _,
            is_column_even,
            is_row_even,
        }: PixelInfo,
        _: &[u16],
        _: usize,
    ) -> (i32, i32, i32) {
        match (is_column_even, is_row_even) {
            (true, true) => (0, v, 0),
            (false, true) => (0, 0, v),
            (true, false) => (v, 0, 0),
            (false, false) => (0, v, 0),
        }
    }
}
