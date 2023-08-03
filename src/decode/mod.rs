use erreport::Report;
use std::io::{Read, Seek};

use crate::{parse::DecodingInfo, tool};

pub(crate) mod arw;

/// These three traits represent three processes needed to decode: ParseExif -> Decode compressed bytes -> Preprocess of image
pub(crate) trait Parse<Info> {
    fn parse_exif<T: Read + Seek>(reader: T) -> Result<Info, Report>;
    fn get_strip_info(&self) -> (u64, usize);
}
pub(crate) trait Decode<Info> {
    fn decode_with_preprocess(&self, strip_bytes: Box<[u8]>) -> Result<Box<[u16]>, Report>;
    fn to_decoding_info(self) -> DecodingInfo;
}
trait Preprocess {
    fn preprocess_over_slice<D: AsMut<[u16]>>(&self, mut data: D) {
        data.as_mut().iter_mut().for_each(|v| {
            *v = self.bl_then_wl(*v);
        });
    }
    fn bl_then_wl(&self, x: u16) -> u16 {
        self.white_level_scaleup(self.black_level_substract(x))
    }
    fn black_level_substract(&self, x: u16) -> u16;
    fn white_level_scaleup(&self, x: u16) -> u16;
}

/// uncompressed 14bit/16bit data in 16bit form
fn general_16bit_iter(image_bytes: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    image_bytes
        .chunks_exact(2)
        .map(move |bytes| tool::u16(bytes, is_le))
}
