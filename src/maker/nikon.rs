use super::*;
use once_cell::sync::Lazy;

use super::utility::GetNumFromBytes;

use super::{
    decode_utility::bit_pump::*, decode_utility::byte_stream::*, decode_utility::huffman::*,
    decode_utility::lookup_table::*, utility::*,
};

pub(super) struct General {
    info: quickexif::ParsedInfo,
}

pub(super) static THUMBNAIL_RULE: Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    quickexif::describe_rule!(tiff {
        0x0112 : u16 / orientation
        0x014a {
            offset address {
                0x0201 / thumbnail
                0x0202 / thumbnail_len
            }
        }
    })
});

pub(super) static IMAGE_RULE: Lazy<quickexif::ParsingRule> = Lazy::new(|| {
    quickexif::describe_rule!(tiff {
        0x0112 : u16 / orientation
        0x8769 {
            0xa302 {
                u32 + 1 / cfa_pattern
            }
            0x927c / maker_notes {
                offset + 18 {
                    0x000c {
                        offset + maker_notes {
                            offset + 10 {
                                r64 + 0 / white_balance_r
                                r64 + 1 / white_balance_b
                                r64 + 2 / white_balance_g
                            }
                        }
                    }
                    0x003d? {
                        offset + maker_notes {
                            offset + 10 {
                                u16 + 0 / black_level
                            }
                        }
                    }
                    0x0045? {
                        offset + maker_notes {
                            offset + 10 {
                                u16 + 0 / crop_left
                                u16 + 1 / crop_top
                                u16 + 2 / crop_width
                                u16 + 3 / crop_height
                            }
                        }
                    }
                    0x008c / contrast_curve_offset(contrast_curve_len)
                    0x0096? / linear_table_offset(linear_table_len)
                }
            }
        }
        0x014a {
            offset + 4 {
                offset address {
                    0x0100 / width
                    0x0101 / height
                    0x0102 : u16 / bps
                    0x0103 : u16 / compression
                    0x0111 / strip
                    0x0117 / strip_len
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
        let width = self.info.u32("crop_width").ok()?;
        let height = self.info.u32("crop_height").ok()?;

        Some(Crop {
            x,
            y,
            width,
            height,
        })
    }
    fn get_white_balance(&self) -> Result<[i32; 3], DecodingError> {
        let r = 512.0 * self.info.f64("white_balance_r")?;
        let g = 512.0 * self.info.f64("white_balance_g")?;
        let b = 512.0 * self.info.f64("white_balance_b")?;
        Ok([r as i32, g as i32, b as i32])
    }
    fn get_thumbnail<'a>(&self, buffer: &'a [u8]) -> Result<&'a [u8], DecodingError> {
        let offset = self.info.usize("thumbnail")?;
        let len = self.info.usize("thumbnail_len")?;
        Ok(&buffer[offset..offset + len])
    }
    fn decode_with_preprocess(&self, buffer: &[u8]) -> Result<Vec<u16>, DecodingError> {
        let strip_offset = self.info.usize("strip")?;
        let strip_len = self.info.usize("strip_len")?;
        let width = self.info.usize("width")?;
        let height = self.info.usize("height")?;
        let bps = self.info.u16("bps")?;
        let bps_scale = self.get_bps_scale()?;
        let compression = self.info.u16("compression")?;
        let black_level = self.info.u16("black_level").unwrap_or(0);
        let black_level = match bps {
            12 => black_level / 4,
            _ => black_level,
        };
        let maker_notes_addr = self.info.usize("maker_notes")? + 10;
        let color_data = match self.info.usize("linear_table_offset") {
            Ok(offset) => {
                let offset = offset + maker_notes_addr;
                let len = self.info.usize("linear_table_len")?;
                &buffer[offset..offset + len]
            }
            Err(_) => {
                let offset = self.info.usize("contrast_curve_offset")? + maker_notes_addr;
                let len = self.info.usize("contrast_curve_len")?;
                &buffer[offset..offset + len]
            }
        };

        let buf = &buffer[strip_offset..];

        macro_rules! to_image {
            ($iter:expr) => {
                $iter
                    .map(|x| bps_scale.saturating_mul(x.saturating_sub(black_level)))
                    .collect()
            };
        }
        let image: Vec<u16> = if width * height * 3 == strip_len {
            let wb_r = self.info.f64("white_balance_r")?;
            let wb_b = self.info.f64("white_balance_b")?;
            to_image!(load_raw_yuv2(buf, wb_r, wb_b, width, height).iter())
        } else {
            match compression {
                1 => match bps {
                    12 => to_image!(to_12bit_iter(buf, self.info.is_le)),
                    14 => to_image!(to_14bit_iter(buf, self.info.is_le)),
                    _ => to_image!(to_16bit_iter(buf, self.info.is_le)),
                },
                0x8799 => {
                    to_image!(
                        load_raw(buf, color_data, self.info.is_le, bps, width, height)?.iter()
                    )
                }
                _ => unimplemented!(),
            }
        };

        if image.len() != width * height {
            Err(DecodingError::InvalidDecodedImageSize(
                image.len(),
                width * height,
            ))
        } else {
            Ok(image)
        }
    }
}

fn load_raw_yuv2(src: &[u8], wb_r: f64, wb_b: f64, width: usize, height: usize) -> Vec<u16> {
    let inv_wb_r = (1024.0 / wb_r) as i32;
    let inv_wb_b = (1024.0 / wb_b) as i32;

    let snef_curve = {
        let g: f32 = 2.4;
        let f: f32 = 0.055;
        let min: f32 = 0.04045;
        let mul: f32 = 12.92;
        let curve = (0..4096)
            .map(|i| {
                let v = (i as f32) / 4095.0;
                let res = if v <= min {
                    v / mul
                } else {
                    ((v + f) / (1.0 + f)).powf(g)
                };
                clampbits((res * 65535.0 * 4.0) as i32, 16)
            })
            .collect::<Vec<u16>>();
        LookupTable::new(&curve)
    };

    let mut out = vec![0u16; width * height];
    out.chunks_mut(width * 3)
        .enumerate()
        .for_each(|(row, out)| {
            let inb = &src[row * width * 3..];
            let mut random = inb.u32be(0);
            for (o, i) in out.chunks_exact_mut(6).zip(inb.chunks_exact(6)) {
                let g1: u16 = i[0] as u16;
                let g2: u16 = i[1] as u16;
                let g3: u16 = i[2] as u16;
                let g4: u16 = i[3] as u16;
                let g5: u16 = i[4] as u16;
                let g6: u16 = i[5] as u16;

                let y1 = (g1 | ((g2 & 0x0f) << 8)) as f32;
                let y2 = ((g2 >> 4) | (g3 << 4)) as f32;
                let cb = (g4 | ((g5 & 0x0f) << 8)) as f32 - 2048.0;
                let cr = ((g5 >> 4) | (g6 << 4)) as f32 - 2048.0;

                let r = snef_curve.dither(clampbits((y1 + 1.370705 * cr) as i32, 12), &mut random);
                let g = snef_curve.dither(
                    clampbits((y1 - 0.337633 * cb - 0.698001 * cr) as i32, 12),
                    &mut random,
                );
                let b = snef_curve.dither(clampbits((y1 + 1.732446 * cb) as i32, 12), &mut random);
                // invert the white balance
                o[0] = clampbits((inv_wb_r * r as i32 + (1 << 9)) >> 10, 15);
                o[1] = g;
                o[2] = clampbits((inv_wb_b * b as i32 + (1 << 9)) >> 10, 15);

                let r = snef_curve.dither(clampbits((y2 + 1.370705 * cr) as i32, 12), &mut random);
                let g = snef_curve.dither(
                    clampbits((y2 - 0.337633 * cb - 0.698001 * cr) as i32, 12),
                    &mut random,
                );
                let b = snef_curve.dither(clampbits((y2 + 1.732446 * cb) as i32, 12), &mut random);
                // invert the white balance
                o[3] = clampbits((inv_wb_r * r as i32 + (1 << 9)) >> 10, 15);
                o[4] = g;
                o[5] = clampbits((inv_wb_b * b as i32 + (1 << 9)) >> 10, 15);
            }
        });

    out
}

fn load_raw(
    src: &[u8],
    meta: &[u8],
    is_le: bool,
    bps: u16,
    width: usize,
    height: usize,
) -> Result<Vec<u16>, DecodingError> {
    let mut out = vec![0u16; width * height];
    let mut stream = ByteStream::new(meta, is_le);
    let v0 = stream.get_u8();
    let v1 = stream.get_u8();

    let mut huff_select = 0;
    if v0 == 73 || v1 == 88 {
        stream.consume_bytes(2110);
    }
    if v0 == 70 {
        huff_select = 2;
    }
    if bps == 14 {
        huff_select += 3;
    }

    // Create the huffman table used to decode
    let mut htable = create_hufftable(huff_select);

    // Setup the predictors
    let mut pred_up1: [i32; 2] = [stream.get_u16() as i32, stream.get_u16() as i32];
    let mut pred_up2: [i32; 2] = [stream.get_u16() as i32, stream.get_u16() as i32];

    // Get the linearization curve
    let mut points = [0u16; 1 << 16];
    for (i, point) in points.iter_mut().enumerate() {
        *point = i as u16;
    }
    let mut max = 1 << bps;
    let csize = stream.get_u16() as usize;
    let mut split = 0usize;
    let step = if csize > 1 { max / (csize - 1) } else { 0 };
    if v0 == 68 && v1 == 32 && step > 0 {
        for i in 0..csize {
            points[i * step] = stream.get_u16();
        }
        for i in 0..max {
            points[i] = ((points[i - i % step] as usize * (step - i % step)
                + points[i - i % step + step] as usize * (i % step))
                / step) as u16;
        }
        split = meta.u16(is_le, 562) as usize;
    } else if v0 != 70 && csize <= 0x4001 {
        for point in points.iter_mut().take(csize) {
            *point = stream.get_u16();
        }
        max = csize;
    }
    let curve = LookupTable::new(&points[0..max]);

    let mut pump = BitPumpMSB::new(src);
    let mut random = pump.peek_bits(24);

    let bps: u32 = bps as u32;
    for row in 0..height {
        if split > 0 && row == split {
            htable = create_hufftable(huff_select + 1);
        }
        pred_up1[row & 1] += htable.huff_decode(&mut pump);
        pred_up2[row & 1] += htable.huff_decode(&mut pump);
        let mut pred_left1 = pred_up1[row & 1];
        let mut pred_left2 = pred_up2[row & 1];
        for col in (0..width).step_by(2) {
            if col > 0 {
                pred_left1 += htable.huff_decode(&mut pump);
                pred_left2 += htable.huff_decode(&mut pump);
            }
            out[row * width + col] = curve.dither(clampbits(pred_left1, bps), &mut random);
            out[row * width + col + 1] = curve.dither(clampbits(pred_left2, bps), &mut random);
        }
    }
    Ok(out)
}

#[inline(always)]
fn clampbits(val: i32, bits: u32) -> u16 {
    let max = (1 << bits) - 1;
    if val < 0 {
        0
    } else if val > max {
        max as u16
    } else {
        val as u16
    }
}

fn create_hufftable(num: usize) -> HuffTable {
    let mut htable = HuffTable::empty();

    for i in 0..15 {
        htable.bits[i] = NIKON_TREE[num][0][i] as u32;
        htable.huffval[i] = NIKON_TREE[num][1][i] as u32;
        htable.shiftval[i] = NIKON_TREE[num][2][i] as u32;
    }

    htable.initialize();
    htable
}

const NIKON_TREE: [[[u8; 16]; 3]; 6] = [
    [
        // 12-bit lossy
        [0, 0, 1, 5, 1, 1, 1, 1, 1, 1, 2, 0, 0, 0, 0, 0],
        [5, 4, 3, 6, 2, 7, 1, 0, 8, 9, 11, 10, 12, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    [
        // 12-bit lossy after split
        [0, 0, 1, 5, 1, 1, 1, 1, 1, 1, 2, 0, 0, 0, 0, 0],
        [6, 5, 5, 5, 5, 5, 4, 3, 2, 1, 0, 11, 12, 12, 0, 0],
        [3, 5, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    [
        // 12-bit lossless
        [0, 0, 1, 4, 2, 3, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0],
        [5, 4, 6, 3, 7, 2, 8, 1, 9, 0, 10, 11, 12, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    [
        // 14-bit lossy
        [0, 0, 1, 4, 3, 1, 1, 1, 1, 1, 2, 0, 0, 0, 0, 0],
        [5, 6, 4, 7, 8, 3, 9, 2, 1, 0, 10, 11, 12, 13, 14, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    [
        // 14-bit lossy after split
        [0, 0, 1, 5, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0, 0, 0],
        [8, 7, 7, 7, 7, 7, 6, 5, 4, 3, 2, 1, 0, 13, 14, 0],
        [0, 5, 4, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    [
        // 14-bit lossless
        [0, 0, 1, 4, 2, 2, 3, 1, 2, 0, 0, 0, 0, 0, 0, 0],
        [7, 6, 8, 5, 9, 4, 10, 3, 11, 12, 2, 0, 1, 13, 14, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
];
