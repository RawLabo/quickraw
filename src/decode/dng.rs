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
                let mut image = self.decode_lossless_rgb(&mut reader).to_report()?;
                for pixel in image.iter_mut() {
                    *pixel = self.bl_then_wl(*pixel);
                }
                Ok(image.into_boxed_slice())
            }
            (7, _) => {
                // lossless compressed bayer
                let mut image = self.decode_lossless_bayer(&mut reader).to_report()?;
                for pixel in image.iter_mut() {
                    *pixel = self.bl_then_wl(*pixel);
                }
                Ok(image.into_boxed_slice())
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
    fn decompress_jpeg_bayer(
        &self,
        buffer: &[u8],
        target: &mut [u16],
        (tile_index, tiles_per_row, last_tile_width, last_tile_height): (
            usize,
            usize,
            usize,
            usize,
        ),
    ) -> Result<(), Report> {
        let tile_width = self.tile_width as usize;
        let tile_height = self.tile_len as usize;
        let processing_row_num = tile_index / tiles_per_row * tile_height;
        let tile_y_offset = processing_row_num * self.width;
        let tile_x_offset = tile_index % tiles_per_row * tile_width;
        let start_offset = tile_y_offset + tile_x_offset;

        let actual_tile_width = if (tile_index + 1) % tiles_per_row == 0 {
            last_tile_width
        } else {
            tile_width
        };
        let actual_tile_height = if processing_row_num + tile_height >= self.height {
            last_tile_height
        } else {
            tile_height
        };

        let jpeg = quickexif::jpeg::JPEG::new(&buffer).to_report()?;
        let mut bit_reader = BitReader::new(jpeg.sos.body);
        let base = 1 << (jpeg.sof.precision - 1);

        // lossless jpeg data in dng for bayer sensor are mostly encoded with two components
        let huffman0 = huffman::HuffmanDecoder::from_dht(&jpeg.dht[0]);
        let huffman1 = huffman::HuffmanDecoder::from_dht(&jpeg.dht[1]);

        let mut diffs = vec![0i32; tile_width * tile_height];
        for pixels in diffs.chunks_exact_mut(2) {
            pixels[0] = huffman0.read_next(&mut bit_reader);
            pixels[1] = huffman1.read_next(&mut bit_reader);
        }

        // cache for first and current two values in row
        let mut cache = [base; 4];
        for tile_row in 0..actual_tile_height {
            cache[0] += diffs[tile_row * tile_width];
            cache[1] += diffs[tile_row * tile_width + 1];
            cache[2] = cache[0];
            cache[3] = cache[1];

            let offset = start_offset + tile_row * self.width;
            target[offset] = cache[2] as u16;
            target[offset + 1] = cache[3] as u16;

            for tile_col in (2..actual_tile_width).step_by(2) {
                cache[2] += diffs[tile_row * tile_width + tile_col];
                cache[3] += diffs[tile_row * tile_width + tile_col + 1];

                let offset = start_offset + tile_row * self.width + tile_col;
                target[offset] = cache[2] as u16;
                target[offset + 1] = cache[3] as u16;
            }
        }

        Ok(())
    }
    fn decode_lossless_bayer<RS: Read + Seek>(&self, mut reader: RS) -> Result<Vec<u16>, Report> {
        let mut image = vec![0u16; self.width * self.height];

        let tile_width = self.tile_width as usize;
        let tile_height = self.tile_len as usize;
        let tiles_per_row = self.width / tile_width + (self.width % tile_width != 0) as usize;
        let tiles_per_col = self.height / tile_height + (self.height % tile_height != 0) as usize;
        let last_tile_width = self.width + tile_width - tile_width * tiles_per_row;
        let last_tile_height = self.height + tile_height - tile_height * tiles_per_col;

        for (tile_index, (&addr, &size)) in self
            .tile_offsets
            .into_iter()
            .zip(self.tiles_sizes.into_iter())
            .enumerate()
        {
            let buffer = get_bytes(&mut reader, addr as u64, size as usize).to_report()?;
            self.decompress_jpeg_bayer(
                &buffer,
                &mut image,
                (tile_index, tiles_per_row, last_tile_width, last_tile_height),
            )
            .to_report()?;
        }

        Ok(image)
    }

    fn decompress_jpeg_rgb(
        &self,
        buffer: &[u8],
        target: &mut [u16],
        (tile_index, tiles_per_row, last_tile_width, last_tile_height): (
            usize,
            usize,
            usize,
            usize,
        ),
    ) -> Result<(), Report> {
        let tile_width = self.tile_width as usize;
        let tile_height = self.tile_len as usize;
        let processing_row_num = tile_index / tiles_per_row * tile_height;
        let tile_y_offset = processing_row_num * self.width * 3;
        let tile_x_offset = tile_index % tiles_per_row * tile_width * 3;
        let start_offset = tile_y_offset + tile_x_offset;

        let actual_tile_width = if (tile_index + 1) % tiles_per_row == 0 {
            last_tile_width
        } else {
            tile_width * 3
        };
        let actual_tile_height = if processing_row_num + tile_height >= self.height {
            last_tile_height
        } else {
            tile_height
        };

        let jpeg = quickexif::jpeg::JPEG::new(&buffer).to_report()?;
        let mut bit_reader = BitReader::new(jpeg.sos.body);
        let base = 1 << (jpeg.sof.precision - 1);
        let predictor = jpeg.sos.ss;

        // lossless jpeg data in dng for rgb are mostly encoded with three components
        let huffman0 = huffman::HuffmanDecoder::from_dht(&jpeg.dht[0]);
        let huffman1 = huffman::HuffmanDecoder::from_dht(&jpeg.dht[1]);
        let huffman2 = huffman::HuffmanDecoder::from_dht(&jpeg.dht[2]);

        let mut diffs = vec![0i32; tile_width * tile_height * 3];
        for pixels in diffs.chunks_exact_mut(3) {
            pixels[0] = huffman0.read_next(&mut bit_reader);
            pixels[1] = huffman1.read_next(&mut bit_reader);
            pixels[2] = huffman2.read_next(&mut bit_reader);
        }

        // cache for first and current three values in row
        let mut cache = [base; 6];
        for tile_row in 0..actual_tile_height {
            cache[0] += diffs[tile_row * tile_width * 3];
            cache[1] += diffs[tile_row * tile_width * 3 + 1];
            cache[2] += diffs[tile_row * tile_width * 3 + 2];
            cache[3] = cache[0];
            cache[4] = cache[1];
            cache[5] = cache[2];

            let offset = start_offset + tile_row * self.width * 3;
            target[offset] = cache[3] as u16;
            target[offset + 1] = cache[4] as u16;
            target[offset + 2] = cache[5] as u16;

            for tile_col in (3..actual_tile_width).step_by(3) {
                let offset = start_offset + tile_row * self.width * 3 + tile_col;

                match (tile_row, predictor) {
                    (row, 7) if row > 0 => {
                        cache[3] = (cache[3] + target[offset - self.width * 3] as i32) / 2
                            + diffs[tile_row * tile_width * 3 + tile_col];
                        cache[4] = (cache[4] + target[offset + 1 - self.width * 3] as i32) / 2
                            + diffs[tile_row * tile_width * 3 + tile_col + 1];
                        cache[5] = (cache[5] + target[offset + 2 - self.width * 3] as i32) / 2
                            + diffs[tile_row * tile_width * 3 + tile_col + 2];
                    }
                    _ => {
                        cache[3] += diffs[tile_row * tile_width * 3 + tile_col];
                        cache[4] += diffs[tile_row * tile_width * 3 + tile_col + 1];
                        cache[5] += diffs[tile_row * tile_width * 3 + tile_col + 2];
                    }
                }

                target[offset] = cache[3] as u16;
                target[offset + 1] = cache[4] as u16;
                target[offset + 2] = cache[5] as u16;
            }
        }

        Ok(())
    }
    fn decode_lossless_rgb<RS: Read + Seek>(&self, mut reader: RS) -> Result<Vec<u16>, Report> {
        let mut image = vec![0u16; self.width * self.height * 3];

        let tile_width = self.tile_width as usize;
        let tile_height = self.tile_len as usize;
        let tiles_per_row = self.width / tile_width + (self.width % tile_width != 0) as usize;
        let tiles_per_col = self.height / tile_height + (self.height % tile_height != 0) as usize;
        let last_tile_width = (self.width + tile_width - tile_width * tiles_per_row) * 3;
        let last_tile_height = self.height + tile_height - tile_height * tiles_per_col;

        for (tile_index, (&addr, &size)) in self
            .tile_offsets
            .into_iter()
            .zip(self.tiles_sizes.into_iter())
            .enumerate()
        {
            let buffer = get_bytes(&mut reader, addr as u64, size as usize).to_report()?;
            self.decompress_jpeg_rgb(
                &buffer,
                &mut image,
                (tile_index, tiles_per_row, last_tile_width, last_tile_height),
            )
            .to_report()?;
        }

        Ok(image)
    }
    fn decode_lossy_jpeg<RS: Read + Seek>(&self, mut reader: RS) -> Result<Box<[u8]>, Report> {
        let mut image = vec![0u8; self.width * self.height * 3];

        let tile_width = self.tile_width as usize;
        let tile_height = self.tile_len as usize;
        let tiles_per_row = self.width / tile_width + (self.width % tile_width != 0) as usize;
        let blank_width = (tile_width * tiles_per_row - self.width) * 3;

        for (tile_index, (&addr, &size)) in self
            .tile_offsets
            .into_iter()
            .zip(self.tiles_sizes.into_iter())
            .enumerate()
        {
            let buffer = get_bytes(&mut reader, addr as u64, size as usize).to_report()?;
            let mut decoder = zune_jpeg::JpegDecoder::new(&buffer);
            let tile_rgb = decoder.decode().to_report()?;

            let processing_row_num = tile_index / tiles_per_row * tile_height;
            let tile_y_offset = processing_row_num * self.width * 3;
            let tile_x_offset = tile_index % tiles_per_row * tile_width * 3;

            for (row_index, data) in tile_rgb.chunks_exact(tile_width * 3).enumerate() {
                if processing_row_num + row_index >= self.height {
                    break;
                }

                let start = row_index * self.width * 3 + tile_x_offset + tile_y_offset;

                if (tile_index + 1) % tiles_per_row == 0 {
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
