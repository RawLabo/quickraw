mod interp;
mod raw_image;
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

#[derive(Debug)]
pub(super) enum CFAPattern {
    RGGB,
    GRBG,
    GBRG,
    BGGR,
    XTrans0, // RBGBRG
    XTrans1, // GGRGGB
}
pub(super) enum Orientation {
    Horizontal,
    Rotate90,
    Rotate180,
    Rotate270,
}
pub(super) struct Crop {
    pub(super) x: u32,
    pub(super) y: u32,
    pub(super) width: u32,
    pub(super) height: u32,
}
pub(super) struct RawImage {
    cfa_pattern: CFAPattern,
    width: usize,
    height: usize,
    pub(super) crop: Option<Crop>,
    pub(super) orientation: Orientation,
    image: Vec<u16>,
    pub(super) white_balance: [i32;3],
    pub(super) cam_matrix: [f32;9]
}
