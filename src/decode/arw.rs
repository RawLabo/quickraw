use std::io::{Read, Seek};

use super::{general_16bit_iter, Decode, Preprocess};
use crate::{
    parse::{self, arw::ArwInfo, get_bytes, DecodingInfo},
    tool::bit_reader::BitReader,
    Error, ToReport,
};
use erreport::Report;

impl Preprocess for ArwInfo {
    fn black_level_substract(&self, x: u16) -> u16 {
        x.saturating_sub(self.black_level)
    }
    fn white_level_scaleup(&self, x: u16) -> u16 {
        x << self.scaleup_factor
    }
}

impl Decode for ArwInfo {
    fn to_decoding_info(self) -> DecodingInfo {
        DecodingInfo {
            width: self.width,
            height: self.height,
            white_balance: self.white_balance,
            cfa_pattern: self.cfa_pattern,
        }
    }
    fn decode_with_preprocess<RS: Read + Seek>(
        &self,
        mut reader: RS,
    ) -> Result<Box<[u16]>, Report> {
        let strip_bytes = get_bytes(&mut reader, self.strip_addr, self.strip_size).to_report()?;
        match self.compression {
            1 => {
                let image = general_16bit_iter(&strip_bytes, self.is_le)
                    .map(|v| self.bl_then_wl(v))
                    .collect();
                Ok(image)
            }
            32767 => self.decompress_32767(&strip_bytes),
            c => Err(Error::UnknownCompression(c)).to_report(),
        }
    }
}

impl ArwInfo {
    /// |<- 32 bytes ->|
    /// |abababababababababababababababab|
    /// |max(11bits) min(11bits) max_index(4bits) min_index(4bits) min_based_offset_values(7bits * 14)|
    /// if max - min > 128(7bits) { min_based_offset_values needs to be scaled up }
    fn decompress_32767(&self, src: &[u8]) -> Result<Box<[u16]>, Report> {
        let w = self.width;
        let mut image = vec![0u16; w * self.height];

        for (row_index, row) in image.chunks_exact_mut(w).enumerate() {
            let mut reader = BitReader::new(&src[row_index * w..]);

            for seg in row.chunks_exact_mut(32) {
                for skip in 0..=1 {
                    let max = reader.read_bits_le(11).to_report()?;
                    let min = reader.read_bits_le(11).to_report()?;
                    let max_index = reader.read_bits_le(4).to_report()? as usize;
                    let min_index = reader.read_bits_le(4).to_report()? as usize;
                    let scale = 32 - ((max - min) >> 7).leading_zeros(); // max scale is 4bits

                    for (i, v) in seg.iter_mut().skip(skip).step_by(2).enumerate() {
                        let val = if i == max_index {
                            max
                        } else if i == min_index {
                            min
                        } else {
                            min + (reader.read_bits_le(7).to_report()? << scale)
                        };

                        // val(11bits) needs to be scale up to 12bits
                        *v = self.bl_then_wl(self.tone_curve[(val << 1) as usize]);
                    }
                }
            }
        }

        Ok(image.into_boxed_slice())
    }
}
