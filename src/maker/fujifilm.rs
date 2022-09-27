use super::*;
use once_cell::sync::Lazy;

pub static FUJI_SENSOR_TABLE: phf::Map<&'static str, u8> = phf::phf_map! {
    "X-T1" => 0, // RBGBRG by default

    "X-T3" => 1, // GGRGGB
    "X-T4" => 1,
    "X-T30" => 1,
    "X-S10" => 1,
    "X-Pro3" => 1,
    "X-Pro4" => 1,
    "X-E4" => 1,
    "X100V" => 1,

    "GFX50R" => 100, // RGGB
    "GFX50S" => 100,
    "GFX100" => 100,
    "GFX50SII" => 100,
    "GFX100S" => 100,
};

pub struct General {
    info: quickexif::ParsedInfo,
}

pub(crate) static THUMBNAIL_RULE: Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    quickexif::describe_rule!(tiff {
        0x0112 / orientation
        next {
            0x0201 / thumbnail
            0x0202 / thumbnail_len
        }
    })
});

pub(crate) static IMAGE_RULE: Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    quickexif::describe_rule!(tiff {
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
    })
});


impl RawDecoder for General {
    fn new(info: quickexif::ParsedInfo) -> Self {
        General { info }
    }
    fn get_info(&self) -> &quickexif::ParsedInfo {
        &self.info
    }
    fn get_crop(&self) -> Option<Crop> {
        None
    }
    fn get_cfa_pattern(&self) -> Result<CFAPattern, DecodingError> {
        let model = self
            .info
            .str("model")?
            .split_whitespace()
            .collect::<String>();
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

