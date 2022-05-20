use super::*;

impl RawDecoder for FujifilmGeneral {
    fn new(info: ParsedRawInfo) -> Self {
        FujifilmGeneral { info }
    }
    fn get_info(&self) -> &ParsedRawInfo {
        &self.info
    }
    fn get_task(only_thumbnail: bool, _model: String) -> ExifTask {
        if only_thumbnail {
            create_rule![tiff {
                0x0112 / orientation
                next {
                    0x0201 / thumbnail
                    0x0202 / thumbnail_len
                }
            }]
        } else {
            create_rule![tiff {
                0x0112 / orientation
                offset + 8 {
                    scan [0x49, 0x49, 0x2a, 0x00] / tiff_offset {
                        tiff {
                            0xf000 {
                                0xf001 / width
                                0xf002 / height
                                0xf003 / bps
                                0xf007 / strip
                                0xf008 / strip_len
                                0xf00a {
                                    u32 + 0 / black_level
                                }
                                0xf00d {
                                    u32 + 0 / white_balance_g
                                    u32 + 1 / white_balance_r
                                    u32 + 2 / white_balance_b
                                }
                            }
                        }
                    }
                }
            }]
        }
    }
    fn get_crop(&self) -> Option<Crop> {
        None
    }
    fn get_cfa_pattern(&self) -> Result<CFAPattern, DecodingError> {
        let model = self.info.str("model")?.split_whitespace().collect::<String>();
        let pattern = FUJI_SENSOR_TABLE.get(model.as_str()).unwrap_or(&0);
        let result = match pattern {
            1 => CFAPattern::XTrans1,
            100 => CFAPattern::RGGB,
            _ => CFAPattern::XTrans0,
        };
        Ok(result)
    }
    fn inner_pre_process(&self, buffer: &[u8]) -> Result<Vec<u16>, DecodingError> {
        let jpeg_header_offset = 12;
        let tiff_offset = self.info.usize("tiff_offset")?;
        let strip_offset = self.info.usize("strip")?;
        let strip_len = self.info.usize("strip_len")?;
        let width = self.info.usize("width")?;
        let height = self.info.usize("height")?;
        let black_level = self.info.u16("black_level")?;
        let bps_scale = self.get_bps_scale()?;

        let data_offset = jpeg_header_offset + tiff_offset + strip_offset;
        let buf = &buffer[data_offset..data_offset + strip_len];
        let image: Vec<u16> = utility::to_14bit_iter(buf, self.info.is_le)
            .map(|x| bps_scale.saturating_mul(x.saturating_sub(black_level)))
            .collect();

        if image.len() != width * height {
            Err(DecodingError::InvalidDecodedImageSize(image.len(), width * height).into())
        } else {
            Ok(image)
        }
    }

    fn inner_get_thumbnail<'a>(&self, buffer: &'a [u8]) -> Result<&'a [u8], DecodingError> {
        let offset = self.info.usize("thumbnail")?;
        let len = self.info.usize("thumbnail_len")?;
        let jpeg_header_offset = 12;
        let tiny_thumbnail_offset = jpeg_header_offset + offset + len;

        let jpeg_eoi = &buffer[tiny_thumbnail_offset..]
            .windows(2)
            .enumerate()
            .find(|(_, data)| data == &[0xff, 0xd9]);

        match jpeg_eoi {
            None => Ok(&buffer[offset..tiny_thumbnail_offset]),
            &Some((index, _)) => Ok(&buffer[..tiny_thumbnail_offset + index + 2]),
        }
    }
}
