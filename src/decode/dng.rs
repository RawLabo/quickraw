use super::{general_16bit_iter, Decode, Preprocess};
use crate::{
    parse::{
        dng::{DngError, DngInfo},
        get_bytes, DecodingInfo,
    },
    ToReport,
};
use erreport::Report;
use std::io::{Read, Seek};

impl Preprocess for DngInfo {
    fn black_level_substract(&self, x: u16) -> u16 {
        x.saturating_sub(self.black_level)
    }
    fn white_level_scaleup(&self, x: u16) -> u16 {
        x << self.scaleup_factor
    }
}

impl Decode for DngInfo {
    fn to_decoding_info(self) -> DecodingInfo {
        DecodingInfo {
            width: self.width,
            height: self.height,
            white_balance: self.white_balance,
            cfa_pattern: self.cfa_pattern,
            color_matrix: Some(self.color_matrix_2),
        }
    }
    fn decode_with_preprocess<RS: Read + Seek>(
        &self,
        mut reader: RS,
    ) -> Result<Box<[u16]>, Report> {
        match (self.compression, self.cfa_pattern.as_ref()) {
            (1, _) => {
                // uncompressed bayer or uncompressed rgb
                let strip_bytes =
                    get_bytes(&mut reader, self.strip_addr, self.strip_size).to_report()?;
                let image = general_16bit_iter(&strip_bytes, self.is_le)
                    .map(|v| self.bl_then_wl(v))
                    .collect();
                Ok(image)
            }
            (7, None) => {
                // lossless compressed rgb
                todo!()
            }
            (7, _) => {
                // lossless compressed bayer
                todo!()
            }
            (34892, _) => {
                // lossy JPEG
                let image = self.decode_lossy_jpeg(&mut reader).to_report()?;
                Ok(image
                    .into_iter()
                    .map(|&x| ((x as u16 + 1) << 8).wrapping_sub(1))
                    .collect())
            }
            _ => Err(DngError::CompressionTypeNotSupported(self.compression)).to_report(),
        }
    }
}

impl DngInfo {
    fn decode_lossy_jpeg<RS: Read + Seek>(&self, mut reader: RS) -> Result<Box<[u8]>, Report> {
        let mut image = vec![0u8; self.width * self.height * 3];

        let tile_width = self.tile_width as usize;
        let tile_height = self.tile_len as usize;
        let tile_per_row = self.width / tile_width + (self.width % tile_width != 0) as usize;
        let blank_width = (tile_width * tile_per_row - self.width) * 3;

        for (tile_index, (&addr, &size)) in self
            .tile_offsets
            .into_iter()
            .zip(self.tile_byte_counts.into_iter())
            .enumerate()
        {
            let buffer = get_bytes(&mut reader, addr as u64, size as usize).to_report()?;
            let mut decoder = zune_jpeg::JpegDecoder::new(&buffer);
            let tile_rgb = decoder.decode().to_report()?;

            let processed_rows = tile_index / tile_per_row * tile_height;
            let tile_y_offset = processed_rows * self.width * 3;
            let tile_x_offset = tile_index % tile_per_row * tile_width * 3;

            for (row_index, data) in tile_rgb.chunks_exact(tile_width * 3).enumerate() {
                if processed_rows + row_index >= self.height {
                    continue;
                }

                let start = row_index * self.width * 3 + tile_x_offset + tile_y_offset;

                if (tile_index + 1) % tile_per_row == 0 {
                    (&mut image[start..start + tile_width * 3 - blank_width])
                        .copy_from_slice(&data[..tile_width * 3 - blank_width]);
                } else {
                    (&mut image[start..start + tile_width * 3]).copy_from_slice(data);
                }
            }
        }

        Ok(image.into_boxed_slice())
    }
}
