use super::*;
use crate::{RawJob, maker::DecodingError};

impl RawImage {
    #[attrs::bench(decoding_with_preprocess)]
    pub(crate) fn new(
        RawJob {
            file_buffer, decoder, ..
        }: &RawJob,
    ) -> Result<Self, DecodingError> {
        let image = decoder.pre_process(file_buffer)?;
        let info = decoder.get_info();
        let cfa_pattern = decoder.get_cfa_pattern()?;
        let orientation = decoder.get_orientation();
        let crop = decoder.get_crop();

        let width = info.usize("width")?;
        let height = info.usize("height")?;

        Ok(RawImage {
            cfa_pattern,
            width,
            height,
            crop,
            orientation,
            image,
        })
    }
}
