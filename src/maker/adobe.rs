use super::*;
use once_cell::sync::Lazy;

use super::utility::GetNumFromBytes;
use super::{decode_utility::ljpeg::LjpegDecompressor, utility::*};

pub(super) struct General {
    info: quickexif::ParsedInfo,
}

pub(super) static THUMBNAIL_RULE: Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    quickexif::describe_rule!(tiff {
        0x0112 : u16 / orientation
        0x014a? / sub_ifd(sub_ifd_count)
        0x828e? / cfa_pattern
        if sub_ifd_count ?
        {
            if sub_ifd_count > 2
            {
                0x014a {
                    offset + 8 {
                        offset address {
                            0x0111 / thumbnail
                            0x0117 / thumbnail_len
                        }
                    }
                }
            }
            else
            {
                if sub_ifd_count > 1
                {
                    0x014a {
                        offset + 4 {
                            offset address {
                                0x0111 / thumbnail
                                0x0117 / thumbnail_len
                            }
                        }
                    }
                }
            }
        }
        if cfa_pattern ? {

        } else {
            0x0100 : u16 / orientation // use width tag to force Horizontal orientation
            0x0111 / thumbnail
            0x0117 / thumbnail_len
        }
    })
});

pub(super) static IMAGE_RULE: Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    let template_rule = quickexif::describe_rule!(template {
        0x0100 / width
        0x0101 / height
        0x0102 : u16 / bps
        0x0103 : u16 / compression
        0x828e? / cfa_pattern
        0xc61d / wl(white_level_len)

        if white_level_len == 1
        {
            0xc61d : u16 / white_level
        }
        else
        {
            0xc61d {
                u16 + 0 / white_level
            }
        }
        0xc61a / bl(black_level_len)
        if black_level_len == 1 {
            0xc61a : u16 / black_level
        } else {
            0xc61a {
                r64 + 0 / black_level
            }
        }
        0x0111? / strip(strip_offsets_count)
        if strip ?
        {
            0x0117 / strip_len
        }
        else
        {
            0x0144 / tile_offsets(tile_offsets_count)
            0x0145 / tile_byte_counts
            0x0142 / tile_width
            0x0143 / tile_len
        }
        if is_adobe_dng_converted ? {
            0xc61f? {
                r64 + 0 / crop_x
                r64 + 1 / crop_y
            }
            0xc620? {
                r64 + 0 / crop_width
                r64 + 1 / crop_height
            }
        } else {
            0xc61f? / crop_origin
            0xc620? / crop_size
        }
    });

    quickexif::describe_rule!(tiff {
        0x0112: u16 / orientation
        0x00fe / sub_file_type
        0xc628 {
            r64 + 0 / white_balance_r
            r64 + 1 / white_balance_g
            r64 + 2 / white_balance_b
        }
        0xc717? / is_adobe_dng_converted
        if sub_file_type == 0
        {
            load(template_rule)
        }
        else
        {
            0x014a / sub_ifd(sub_ifd_count)
            if sub_ifd_count == 1
            {
                0x014a {
                    0x00fe / sub_file_type1
                    if sub_file_type1 == 0
                    {
                        load(template_rule)
                    }
                }
            }
            else
            {
                0x014a {
                    offset address {
                        0x00fe / sub_file_type2
                        if sub_file_type2 == 0
                        {
                            load(template_rule)
                        }
                    }
                }
            }
        }
    })
});

impl General {
    fn get_white_level_scale(&self) -> Result<u16, quickexif::parsed_info::Error> {
        let white_level = self.info.u16("white_level")?;
        Ok(u16::MAX / white_level)
    }
}

impl RawDecoder for General {
    fn new(info: quickexif::ParsedInfo) -> Self {
        General { info }
    }
    fn get_info(&self) -> &quickexif::ParsedInfo {
        &self.info
    }
    fn into_info(self) -> quickexif::ParsedInfo {
        self.info
    }
    fn get_white_balance(&self) -> Result<[i32; 3], DecodingError> {
        let r = 512.0 / self.info.f64("white_balance_r")?;
        let g = 512.0 / self.info.f64("white_balance_g")?;
        let b = 512.0 / self.info.f64("white_balance_b")?;
        Ok([r as i32, g as i32, b as i32])
    }
    fn get_crop(&self) -> Option<Crop> {
        if let (Ok(crop_origin), Ok(crop_size)) =
            (self.info.u8a4("crop_origin"), self.info.u8a4("crop_size"))
        {
            let x = crop_origin.as_slice().u16(self.info.is_le, 0) as u32;
            let y = crop_origin.as_slice().u16(self.info.is_le, 2) as u32;
            let width = crop_size.as_slice().u16(self.info.is_le, 0) as u32;
            let height = crop_size.as_slice().u16(self.info.is_le, 2) as u32;
            Some(Crop {
                x,
                y,
                width,
                height,
            })
        } else {
            let x = self.info.f64("crop_x").ok()? as u32;
            let y = self.info.f64("crop_y").ok()? as u32;
            let width = self.info.f64("crop_width").ok()? as u32;
            let height = self.info.f64("crop_height").ok()? as u32;
            Some(Crop {
                x,
                y,
                width,
                height,
            })
        }
    }
    fn get_thumbnail<'a>(&self, buffer: &'a [u8]) -> Result<&'a [u8], DecodingError> {
        let offset = self.info.usize("thumbnail")?;
        let len = self.info.usize("thumbnail_len")?;

        Ok(&buffer[offset..offset + len])
    }
    fn decode_with_preprocess(&self, buffer: &[u8]) -> Result<Vec<u16>, DecodingError> {
        let width = self.info.usize("width")?;
        let height = self.info.usize("height")?;
        let compression = self.info.u16("compression")?;
        let bps = self.info.u16("bps")?;
        let white_level_scale = self.get_white_level_scale()?;
        let black_level = self.info.u16("black_level")?;

        macro_rules! to_image {
            ($iter:expr) => {
                $iter
                    .map(|x| white_level_scale.saturating_mul(x.saturating_sub(black_level)))
                    .collect()
            };
        }

        let image: Vec<u16> = match compression {
            1 => { // uncompressed dng
                let offset_addr = self.info.usize("strip")?;
                let offset_count = self.info.usize("strip_offsets_count")?;
                let len_addr = self.info.usize("strip_len")?;

                let buf = if (offset_count) > 1 {
                    let strip_addr = buffer.u32(self.info.is_le, offset_addr) as usize;
                    let tile_count = buffer.u32(self.info.is_le, len_addr) as usize;
                    &buffer[strip_addr..strip_addr + tile_count * offset_count]
                } else {
                    &buffer[offset_addr..offset_addr + len_addr]
                };

                match bps {
                    12 => to_image!(to_12bit_iter_packed(buf, self.info.is_le)),
                    14 => to_image!(to_14bit_iter_packed(buf, self.info.is_le)),
                    _ => to_image!(to_16bit_iter(buf, self.info.is_le)),
                }
            }
            7 => {
                // only support Apple ProRaw now
                let byte_counts_addr = self.info.usize("tile_byte_counts")?;
                let tile_offsets_count = self.info.usize("tile_offsets_count")?;
                let tile_offsets_addr = self.info.usize("tile_offsets")?;
                let tile_width = self.info.usize("tile_width")?;
                let tile_len = self.info.usize("tile_len")?;

                let offsets_iter =
                    buffer[tile_offsets_addr..tile_offsets_addr + 4 * tile_offsets_count].chunks(4);
                let counts_iter =
                    buffer[byte_counts_addr..byte_counts_addr + 4 * tile_offsets_count].chunks(4);

                let tiles = offsets_iter
                    .zip(counts_iter)
                    .map(|(offset_bytes, count_bytes)| {
                        let offset_addr = offset_bytes.u32(self.info.is_le, 0) as usize;
                        let count_addr = count_bytes.u32(self.info.is_le, 0) as usize;
                        (offset_addr, count_addr)
                    })
                    .collect::<Vec<_>>();

                load_compressed(buffer, width, height, tiles, tile_width, tile_len)?
            }
            _ => {
                unimplemented!()
            }
        };

        if image.len() != width * height && image.len() != width * height * 3 {
            Err(DecodingError::InvalidDecodedImageSize(
                image.len(),
                width * height,
            ))
        } else {
            Ok(image)
        }
    }
}

fn load_compressed(
    buffer: &[u8],
    width: usize,
    height: usize,
    tiles: Vec<(usize, usize)>,
    tile_width: usize,
    tile_height: usize,
) -> Result<Vec<u16>, DecodingError> {
    let mut out = vec![0u16; width * height * 3];

    let tile_count_per_row = width / tile_width;

    for (tile_index, (addr, size)) in tiles.into_iter().enumerate() {
        let col = tile_index % tile_count_per_row * tile_width * 3;
        let row = tile_index / tile_count_per_row * tile_height;

        let mut tile_out = vec![0u16; tile_width * tile_height * 3];

        let src = &buffer[addr..addr + size];
        let decompressor = LjpegDecompressor::new(src)?;

        decompressor.decode(
            &mut tile_out,
            0,
            tile_width * 3,
            tile_width * 3,
            tile_height,
        )?;

        tile_out
            .chunks(tile_width * 3)
            .enumerate()
            .for_each(|(offset_row, data)| {
                let start = col + (row + offset_row) * width * 3;
                let end = start + tile_width * 3;
                (&mut out[start..end]).copy_from_slice(data);
            });
    }

    Ok(out)
}
