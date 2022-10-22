mod interp;
mod renderer;

#[derive(Debug)]
pub struct PixelInfo {
    pub i: usize,
    pub v: i32,
    pub x: usize,
    pub y: usize,
    pub is_top: bool,
    pub is_left: bool,
    pub is_bottom: bool,
    pub is_right: bool,
    pub is_column_even: bool,
    pub is_row_even: bool,
}

impl PixelInfo {
    #[inline(always)]
    pub(crate) fn new(i: usize, v: u16, w: usize, h: usize) -> Self {
        let x = i % w;
        let y = i / w;
        let is_top = y == 0;
        let is_left = x == 0;
        let is_bottom = y == h - 1;
        let is_right = x == w - 1;
        let is_column_even = x % 2 == 0;
        let is_row_even = y % 2 == 0;

        Self {
            i,
            v: v as i32,
            x,
            y,
            is_top,
            is_bottom,
            is_left,
            is_right,
            is_column_even,
            is_row_even,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum CFAPattern {
    RGGB,
    GRBG,
    GBRG,
    BGGR,
    XTrans0, // RBGBRG
    XTrans1, // GGRGGB
}
pub enum Orientation {
    Horizontal,
    Rotate90,
    Rotate180,
    Rotate270,
}
pub struct Crop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
pub struct DecodedImage {
    pub cfa_pattern: CFAPattern,
    pub width: usize,
    pub height: usize,
    pub crop: Option<Crop>,
    pub orientation: Orientation,
    pub image: Vec<u16>,
    pub white_balance: [i32; 3],
    pub cam_matrix: [f32; 9],
    pub scale_factor: i32,
    pub black_level: i32
}
