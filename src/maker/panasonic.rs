use super::decode_utility::bit_pump::*;
use super::*;
use once_cell::sync::Lazy;

pub(super) struct General {
    info: quickexif::ParsedInfo,
}

pub(super) static THUMBNAIL_RULE: Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    quickexif::describe_rule!(tiff {
        0x0112 / orientation
        0x002e / thumbnail(thumbnail_len)
    })
});

pub(super) static IMAGE_RULE: Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    quickexif::describe_rule!(tiff {
        0x0002 / width
        0x0003 / height
        0x0009 / cfa_pattern
        0x000a / bps
        0x001c / black_level_r
        0x001d / black_level_g
        0x001e / black_level_b
        0x0024 / white_balance_r
        0x0025 / white_balance_g
        0x0026 / white_balance_b
        0x0118 / strip
        0x0117 / strip_len
        0x002f? / crop_top
        0x0030? / crop_left
        0x0031? / crop_bottom
        0x0032? / crop_right
        0x0112 / orientation
        0x002e {
            offset + 12 {
                tiff {
                    0x8769 {
                        0x927c {
                            offset + 12 {
                                0x004b / cropped_width
                                0x004c / cropped_height
                            }
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
        let x = self.info.u32("crop_left").ok()?;
        let y = self.info.u32("crop_top").ok()?;
        let right = self.info.u32("crop_right").ok()?;
        let bottom = self.info.u32("crop_bottom").ok()?;

        Some(Crop {
            x,
            y,
            width: right - x,
            height: bottom - y,
        })
    }
    fn inner_pre_process(&self, buffer: &[u8]) -> Result<Vec<u16>, DecodingError> {
        let image = load_raw(&self.info, buffer)?;
        let black_level = self.info.u16("black_level_r")?;
        let bps_scale = self.get_bps_scale()?;
        Ok(image
            .iter()
            .map(|x| bps_scale.saturating_mul(x.saturating_sub(black_level)))
            .collect())
    }
    fn inner_get_cfa_pattern(&self) -> Result<CFAPattern, DecodingError> {
        let cfa_pattern = self.info.u16("cfa_pattern")?;
        let result = match cfa_pattern {
            1 => CFAPattern::RGGB,
            2 => CFAPattern::GRBG,
            3 => CFAPattern::GBRG,
            4 => CFAPattern::BGGR,
            _ => CFAPattern::RGGB,
        };
        Ok(result)
    }
    fn inner_get_thumbnail<'a>(&self, buffer: &'a [u8]) -> Result<&'a [u8], DecodingError> {
        let offset = self.info.usize("thumbnail")?;
        let len = self.info.usize("thumbnail_len")?;
        Ok(&buffer[offset..offset + len])
    }
}

fn load_raw(info: &quickexif::ParsedInfo, buffer: &[u8]) -> Result<Vec<u16>, DecodingError> {
    const SPLIT: bool = true;
    const BLOCK_LINES: usize = 5;

    let width = info.usize("width")?;
    let height = info.usize("height")?;
    let offset = info.usize("strip")?;

    let buf = &buffer[offset..];
    let mut out: Vec<u16> = vec![0u16; width * height];

    out.chunks_exact_mut(width * BLOCK_LINES)
        .enumerate()
        .for_each(|(index, out)| {
            let row = index * BLOCK_LINES;

            let skip = ((width * row * 9) + (width / 14 * 2 * row)) / 8;
            let blocks = skip / 0x4000;
            let src = &buf[blocks * 0x4000..];
            let mut pump = BitPumpPanasonic::new(src, SPLIT);
            for _ in 0..(skip % 0x4000) {
                pump.get_bits(8);
            }

            let mut sh: i32 = 0;
            for out in out.chunks_exact_mut(14) {
                let mut pred: [i32; 2] = [0, 0];
                let mut nonz: [i32; 2] = [0, 0];

                for i in 0..14 {
                    if (i % 3) == 2 {
                        sh = 4 >> (3 - pump.get_bits(2));
                    }
                    if nonz[i & 1] != 0 {
                        let j = pump.get_bits(8) as i32;
                        if j != 0 {
                            pred[i & 1] -= 0x80 << sh;
                            if pred[i & 1] < 0 || sh == 4 {
                                pred[i & 1] &= !(-1 << sh);
                            }
                            pred[i & 1] += j << sh;
                        }
                    } else {
                        nonz[i & 1] = pump.get_bits(8) as i32;
                        if nonz[i & 1] != 0 || i > 11 {
                            pred[i & 1] = nonz[i & 1] << 4 | (pump.get_bits(4) as i32);
                        }
                    }
                    out[i] = pred[i & 1] as u16;
                }
            }
        });
    Ok(out)
}
