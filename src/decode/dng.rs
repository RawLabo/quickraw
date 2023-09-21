use super::{general_16bit_iter, Decode, Preprocess};
use crate::{
    parse::{
        dng::{DngError, DngInfo},
        get_bytes, DecodingInfo,
    },
    tool::{bit_reader::BitReader, huffman},
    ToReport,
};
use erreport::Report;
use std::io::{Read, Seek};
use wide::u32x4;

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
                println!("compressed rgb");
                todo!()
            }
            (7, _) => {
                // lossless compressed bayer
                let image = self.decode_lossless_bayer(&mut reader).to_report()?;
                let image = image.into_iter().map(|&v| self.bl_then_wl(v)).collect();
                Ok(image)
            }
            (34892, _) => {
                // lossy JPEG
                let image = self.decode_lossy_jpeg(&mut reader).to_report()?;
                let x0 = u32x4::from(self.map_polynomial[0]);
                let x1 = u32x4::from(self.map_polynomial[1]);
                let x2 = u32x4::from(self.map_polynomial[2]);
                let x3 = u32x4::from(self.map_polynomial[3]);

                let result = image
                    .chunks_exact(3)
                    .flat_map(|rgb| {
                        let rgb = u32x4::from([rgb[0] as u32, rgb[1] as u32, rgb[2] as u32, 0]);
                        let result: u32x4 =
                            (x0 + x1 * rgb + x2 * rgb * rgb + x3 * rgb * rgb * rgb) >> 16;
                        let &[r, g, b, _] = result.as_array_ref();
                        [r as u16, g as u16, b as u16]
                    })
                    .collect();
                Ok(result)
            }
            _ => Err(DngError::CompressionTypeNotSupported(self.compression)).to_report(),
        }
    }
}

impl DngInfo {
    fn decode_tile(&self, bytes: &[u8]) -> Result<Vec<u16>, Report> {
        let jpeg = quickexif::jpeg::JPEG::new(&bytes).to_report()?;
        let mut bit_reader = BitReader::new(jpeg.sos.body);
        let base = 1 << (jpeg.sof.precision - 1);

        // lossless jpeg data in dng for bayer sensor are mostly encoded with two components
        let huffman0 = huffman::HuffmanDecoder::from_dht(&jpeg.dht[0]);
        let huffman1 = huffman::HuffmanDecoder::from_dht(&jpeg.dht[1]);

        let mut image = vec![0u16; self.tile_width as usize * self.tile_len as usize];
        image[0] = (base + huffman0.read_next(&mut bit_reader)) as u16;
        image[1] = (base + huffman1.read_next(&mut bit_reader)) as u16;

        let tile_width = self.tile_width as usize;
        for index in (2..image.len()).step_by(2) {
            let col = index % tile_width;
            let diff0 = huffman0.read_next(&mut bit_reader);
            let diff1 = huffman1.read_next(&mut bit_reader);

            let (base0, base1) = if col == 0 {
                (image[index - tile_width], image[index + 1 - tile_width])
            } else {
                (image[index - 2], image[index - 1])
            };

            image[index] = (base0 as i32 + diff0) as u16;
            image[index + 1] = (base1 as i32 + diff1) as u16;
        }

        Ok(image)
    }
    fn decode_lossless_bayer<RS: Read + Seek>(&self, mut reader: RS) -> Result<Box<[u16]>, Report> {
        let mut image = vec![0u16; self.width * self.height];

        let tile_width = self.tile_width as usize;
        let tile_height = self.tile_len as usize;
        let tile_per_row = self.width / tile_width + (self.width % tile_width != 0) as usize;
        let blank_width = tile_width * tile_per_row - self.width;

        for (tile_index, (&addr, &size)) in self
            .tile_offsets
            .into_iter()
            .zip(self.tiles_sizes.into_iter())
            .enumerate()
        {
            let buffer = get_bytes(&mut reader, addr as u64, size as usize).to_report()?;
            let tile_image = self.decode_tile(&buffer).to_report()?;

            let processed_rows = tile_index / tile_per_row * tile_height;
            let tile_y_offset = processed_rows * self.width;
            let tile_x_offset = tile_index % tile_per_row * tile_width;

            for (row_index, data) in tile_image.chunks_exact(tile_width).enumerate() {
                if processed_rows + row_index >= self.height {
                    continue;
                }

                let start = row_index * self.width + tile_x_offset + tile_y_offset;

                if (tile_index + 1) % tile_per_row == 0 {
                    (&mut image[start..start + tile_width - blank_width])
                        .copy_from_slice(&data[..tile_width - blank_width]);
                } else {
                    (&mut image[start..start + tile_width]).copy_from_slice(data);
                }
            }
        }

        Ok(image.into_boxed_slice())
    }
    fn decode_lossy_jpeg<RS: Read + Seek>(&self, mut reader: RS) -> Result<Box<[u8]>, Report> {
        let mut image = vec![0u8; self.width * self.height * 3];

        let tile_width = self.tile_width as usize;
        let tile_height = self.tile_len as usize;
        let tile_per_row = self.width / tile_width + (self.width % tile_width != 0) as usize;
        let blank_width = (tile_width * tile_per_row - self.width) * 3;

        for (tile_index, (&addr, &size)) in self
            .tile_offsets
            .into_iter()
            .zip(self.tiles_sizes.into_iter())
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
