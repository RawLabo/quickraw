mod interp;
mod impls;
mod renderer;

#[derive(Debug)]
pub(super) struct PixelInfo {
    i: usize,
    v: i32,
    x: usize,
    y: usize,
    is_top: bool,
    is_left: bool,
    is_bottom: bool,
    is_right: bool,
    is_column_even: bool,
    is_row_even: bool,
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
    pub white_balance: [i32;3],
    pub cam_matrix: [f32;9]
}
