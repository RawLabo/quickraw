use super::{super::utility::to_14bit_iter, decode_utility::bit_pump::*, decode_utility::lookup_table::*, *};
use crate::tiff::utility::GetNumFromBytes;
use std::cmp;

impl SonyGeneral {
    fn get_white_level_scale(&self) -> Result<u16, RawInfoError> {
        let legacy_white_level = self.info.u16("legacy_white_level")?;
        let result = match legacy_white_level {
            15360 => 4,
            _ => {
                let white_level = self.info.u16("white_level")?;
                match white_level {
                    15360 => 4,
                    _ => 1,
                }
            }
        };
        Ok(result)
    }
}

impl RawDecoder for SonyGeneral {
    fn new(info: ParsedRawInfo) -> Self {
        SonyGeneral { info }
    }
    fn get_info(&self) -> &ParsedRawInfo {
        &self.info
    }

    fn get_task(only_thumbnail: bool, _model: String) -> ExifTask {
        if only_thumbnail {
            create_rule![tiff {
                0x0112 / orientation
                0x0201 / preview_offset
                0x0202 / preview_len
            }]
        } else {
            create_rule![tiff {
                0x0112 / orientation
                0x8769 {
                    0x9102 {
                        r64 + 0 / compressed_bps
                    }
                }
                0x014a {
                    0x0103 / compression
                    0x0100 / width
                    0x0101 / height
                    0x0102 / bps
                    0x828e / cfa_pattern
                    0x0111 / strip
                    0x0117 / strip_len
                    0x7010? / tone_curve_addr
                    0xc61f? {
                        u32 + 0 / crop_x
                        u32 + 1 / crop_y
                    }
                    0xc620? {
                        u32 + 0 / crop_w
                        u32 + 1 / crop_h
                    }
                }
                0xc634 {
                    sony_decrypt / 0x7200 / 0x7201 / 0x7221 {
                        0x7310 {
                            u16 + 0 / black_level
                        }
                        0x7312 {
                            u16 + 0 / white_balance_r
                            u16 + 1 / white_balance_g
                            u16 + 3 / white_balance_b
                        }
                        0x787f / legacy_white_level {
                            u16 + 0 / white_level
                        }
                    }
                }
            }]
        }
    }
    fn get_crop(&self) -> Option<Crop> {
        let x = self.info.u32("crop_x").ok()?;
        let y = self.info.u32("crop_y").ok()?;
        let width = self.info.u32("crop_w").ok()?;
        let height = self.info.u32("crop_h").ok()?;

        Some(Crop { x, y, width, height })
    }
    fn inner_pre_process(&self, buffer: &[u8]) -> Result<Vec<u16>, DecodingError> {
        let width = self.info.usize("width")?;
        let height = self.info.usize("height")?;
        let black_level = self.info.u16("black_level")?;
        let strip_offset = self.info.usize("strip")?;
        let strip_len = self.info.usize("strip_len")?;
        let compression = self.info.u32("compression")?;
        let level_scale = self.get_white_level_scale()?;
        let buf = &buffer[strip_offset..strip_offset + strip_len];

        let black_level_sub = |v: u16| level_scale.saturating_mul(v.saturating_sub(black_level));

        let image: Vec<u16> = match compression {
            0x7fffu32 => {
                let tone_curve_addr = self.info.usize("tone_curve_addr")?;
                let tone_curve = buffer[tone_curve_addr..tone_curve_addr + 8]
                    .chunks_exact(2)
                    .map(|x| x.u16(self.info.is_le, 0))
                    .collect::<Vec<u16>>();

                load_raw8(buf, &tone_curve, width, height)
                    .iter()
                    .copied()
                    .map(black_level_sub)
                    .collect()
            }
            7 => unimplemented!(),
            _ => to_14bit_iter(buf, self.info.is_le).map(black_level_sub).collect(),
        };

        if image.len() != width * height {
            Err(DecodingError::InvalidDecodedImageSize(image.len(), width * height).into())
        } else {
            Ok(image)
        }
    }
    fn inner_get_thumbnail<'a>(&self, buffer: &'a [u8]) -> Result<&'a [u8], DecodingError> {
        let offset = self.info.usize("preview_offset")?;
        let len = self.info.usize("preview_len")?;
        Ok(&buffer[offset..offset + len])
    }
}

fn gen_curve_lut(tone_curve: &[u16]) -> LookupTable {
    let mut curve: [usize; 6] = [0, 0, 0, 0, 0, 4095];

    for i in 0..4 {
        curve[i + 1] = ((tone_curve[i] as u32 >> 2) & 0xfff) as usize;
    }

    let mut table = vec![0 as u16; curve[5] + 1];
    for i in 0..5 {
        for j in (curve[i] + 1)..(curve[i + 1] + 1) {
            table[j] = table[(j - 1)] + (1 << i);
        }
    }

    LookupTable::new(&table)
}

fn load_raw8(buf: &[u8], tone_curve: &[u16], width: usize, height: usize) -> Vec<u16> {
    let mut out: Vec<u16> = vec![0u16; width * height];
    let curve = gen_curve_lut(tone_curve);

    out.chunks_exact_mut(width).enumerate().for_each(|(row_index, out)| {
        let mut pump = BitPumpLSB::new(&buf[(row_index * width)..]);

        let mut random = pump.peek_bits(16);
        for out in out.chunks_exact_mut(32) {
            // Process 32 pixels at a time in interleaved fashion
            for j in 0..2 {
                let max = pump.get_bits(11);
                let min = pump.get_bits(11);
                let delta = max - min;
                // Calculate the size of the data shift needed by how large the delta is
                // A delta with 11 bits requires a shift of 4, 10 bits of 3, etc
                let delta_shift: u32 = cmp::max(0, (32 - (delta.leading_zeros() as i32)) - 7) as u32;
                let imax = pump.get_bits(4) as usize;
                let imin = pump.get_bits(4) as usize;

                for i in 0..16 {
                    let val = if i == imax {
                        max
                    } else if i == imin {
                        min
                    } else {
                        cmp::min(0x7ff, (pump.get_bits(7) << delta_shift) + min)
                    };
                    out[j + (i * 2)] = curve.dither((val << 1) as u16, &mut random);
                }
            }
        }
    });
    out
}
