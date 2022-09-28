use super::*;

impl RawImage {
    pub(crate) fn new(
        image: Vec<u16>,
        width: usize,
        height: usize,
        cfa_pattern: CFAPattern,
        crop: Option<Crop>,
        orientation: Orientation,
        (white_balance, cam_matrix): ([i32;3], [f32;9])
    ) -> Self {
        RawImage {
            cfa_pattern,
            width,
            height,
            crop,
            orientation,
            image,
            white_balance,
            cam_matrix
        }
    }
}
