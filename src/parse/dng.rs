use super::{CFAPattern, ColorMatrix, Parse, WhiteBalance};
use crate::{parse::get_scaleup_factor, Error, ToReport};
use erreport::Report;
use std::io::{self, BufReader, Cursor, Read, Seek};

#[derive(thiserror::Error, Debug)]
pub(crate) enum DngError {
    #[error("No subifd blocks contains cfa_pattern.")]
    NoCfaPatternFound,
    #[error("Unsupported compression type: {0}")]
    CompressionTypeNotSupported(u16),
}

mod dng_rule {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0xc717 is_converted
            0xc621 color_matrix_1 // for apple pro raw
            0xc622 color_matrix_2 // for normal dng
            0x0112 orientation
            0xc628 white_balance

            0x0100 width0
            0x0101 height0
            0x0102 bps0
            0x0103 compression0
            0x0111 strip_addr0
            0x0117 strip_size0
            0x828e cfa_pattern0
            0x0144 tile_offsets0
            0x0145 tile_byte_counts0
            0x0142 tile_width0
            0x0143 tile_len0
            0xc61d white_level0
            0xc61a black_level0
        }
        0 -> 0x014a -> 0 {
            0x0100 width1
            0x0101 height1
            0x0102 bps1
            0x0103 compression1
            0x0111 strip_addr1
            0x0117 strip_size1
            0x828e cfa_pattern1
            0x0144 tile_offsets1
            0x0145 tile_byte_counts1
            0x0142 tile_width1
            0x0143 tile_len1
            0xc61d white_level1
            0xc61a black_level1
            0xc741 opcodelist2
        }
        0 -> 0x014a -> 100 {
            0x0100 width2
            0x0101 height2
            0x0102 bps2
            0x0103 compression2
            0x0111 strip_addr2
            0x0117 strip_size2
            0x828e cfa_pattern2
            0x0144 tile_offsets2
            0x0145 tile_byte_counts2
            0x0142 tile_width2
            0x0143 tile_len2
            0xc61d white_level2
            0xc61a black_level2
        }
    );
}

pub struct DngInfo {
    pub is_le: bool,
    pub is_converted: bool,
    pub white_balance: WhiteBalance,
    pub color_matrix_1: ColorMatrix,
    pub color_matrix_2: ColorMatrix,
    pub map_polynomial: [[f32; 4]; 4],

    pub compression: u16,
    pub strip_addr: u64,
    pub strip_size: usize,
    pub tile_offsets: Box<[u32]>,
    pub tile_byte_counts: Box<[u32]>,
    pub tile_width: u32,
    pub tile_len: u32,
    pub thumbnail: Option<(u64, usize)>,

    pub width: usize,
    pub height: usize,
    pub orientation: u16,
    pub cfa_pattern: Option<CFAPattern>,
    pub black_level: u16,
    pub scaleup_factor: u16,
}

impl Parse<DngInfo> for DngInfo {
    fn parse_exif<T: Read + Seek>(mut reader: T) -> Result<DngInfo, Report> {
        let buf_reader = BufReader::new(&mut reader);
        let (exif, is_le) =
            quickexif::parse_exif(buf_reader, dng_rule::PATH_LST, None).to_report()?;

        super::gen_get!(exif, dng_rule);

        let is_converted = get!(is_converted).is_some();
        let color_matrix_1: ColorMatrix = get!(color_matrix_1 => r64s).into();
        let color_matrix_2: ColorMatrix = get!(color_matrix_2 => r64s).into();
        let orientation = get!(orientation, u16);
        let white_balance = get!(white_balance => r64s);
        let white_balance = [
            (1024f64 / white_balance[0]) as u16,
            1024,
            (1024f64 / white_balance[2]) as u16,
        ];

        let mut thumbnail = None;
        // detect if there is subifd-0
        let (compression, tags): (u16, [&(u16, u16); 11]) = if let Some(compression) =
            get!(compression1)
        {
            // has subifd-0
            // detect if there is subifd-1
            if let (Some(thumb_addr), Some(thumb_size)) = (get!(strip_addr2), get!(strip_size2)) {
                thumbnail = Some((thumb_addr.u32() as u64, thumb_size.u32() as usize));
            } else {
                // use the strip in ifd-0 as thumbnail
                thumbnail = Some((
                    get!(strip_addr0, u32) as u64,
                    get!(strip_size0, u32) as usize,
                ));
            }

            (
                compression.u16(),
                [
                    dng_rule::strip_addr1,
                    dng_rule::strip_size1,
                    dng_rule::tile_offsets1,
                    dng_rule::tile_byte_counts1,
                    dng_rule::tile_width1,
                    dng_rule::tile_len1,
                    dng_rule::width1,
                    dng_rule::height1,
                    dng_rule::white_level1,
                    dng_rule::black_level1,
                    dng_rule::cfa_pattern1,
                ],
            )
        } else {
            // no subifd
            (
                get!(compression0, u16),
                [
                    dng_rule::strip_addr0,
                    dng_rule::strip_size0,
                    dng_rule::tile_offsets0,
                    dng_rule::tile_byte_counts0,
                    dng_rule::tile_width0,
                    dng_rule::tile_len0,
                    dng_rule::width0,
                    dng_rule::height0,
                    dng_rule::white_level0,
                    dng_rule::black_level0,
                    dng_rule::cfa_pattern0,
                ],
            )
        };

        let (strip_addr, strip_size, tile_offsets, tile_byte_counts, tile_width, tile_len) =
            match compression {
                1 => {
                    let strip_addr = get!(tags[0], u32) as u64;
                    let strip_size = get!(tags[1], u32) as usize;
                    (strip_addr, strip_size, [].into(), [].into(), 0, 0)
                }
                7 | 34892 => {
                    let tile_offsets = get!(tags[2] => u32s);
                    let tile_byte_counts = get!(tags[3] => u32s);
                    let tile_width = get!(tags[4], u32);
                    let tile_len = get!(tags[5], u32);
                    (0, 0, tile_offsets, tile_byte_counts, tile_width, tile_len)
                }
                _ => return Err(DngError::CompressionTypeNotSupported(compression)).to_report(),
            };

        let width = get!(tags[6], u32) as usize;
        let height = get!(tags[7], u32) as usize;
        let white_level = if let Some(wl) = get!(tags[8]).and_then(|x| x.u16s()) {
            wl[0] as u16
        } else {
            get!(tags[8], u16)
        };
        let black_level = if let Some(bl) = get!(tags[9]).and_then(|x| x.r64s()) {
            bl[0] as u16
        } else {
            get!(tags[9], u16)
        };
        let cfa_pattern = get!(tags[10]).map(|x| x.raw().into());

        let scaleup_factor = get_scaleup_factor(white_level);

        let mut map_polynomial = [[0f32; 4]; 4];
        if let Some(opcodelist) = get!(opcodelist2) {
            let mut reader = Cursor::new(opcodelist.raw());
            let mut op_count = reader.u32().to_report()?;
            let mut plane_id = 0;
            while op_count > 0 {
                let op = reader.u32().to_report()?;
                if op == 8 {
                    // is MapPolynomial
                    reader.shift::<44>().to_report()?;
                    let degree = reader.u32().to_report()?;
                    if degree != 3 {
                        // currently support degree = 3 only
                        continue;
                    }

                    map_polynomial[0][plane_id] = reader.f64().to_report()? as f32;
                    map_polynomial[1][plane_id] = reader.f64().to_report()? as f32;
                    map_polynomial[2][plane_id] = reader.f64().to_report()? as f32;
                    map_polynomial[3][plane_id] = reader.f64().to_report()? as f32;

                    plane_id += 1;
                    if plane_id == 3 {
                        break;
                    }
                }
                op_count -= 1;
            }
        }

        Ok(DngInfo {
            is_le,
            is_converted,
            width,
            height,
            orientation,
            cfa_pattern,
            compression,
            black_level,
            scaleup_factor,
            white_balance: white_balance.into(),
            strip_addr,
            strip_size,
            tile_offsets,
            tile_byte_counts,
            tile_width,
            tile_len,
            thumbnail,
            color_matrix_1,
            color_matrix_2,
            map_polynomial,
        })
    }
}

trait Read4Opcode {
    fn u32(&mut self) -> Result<u32, io::Error>;
    fn f64(&mut self) -> Result<f64, io::Error>;
    fn shift<const N: usize>(&mut self) -> Result<(), io::Error>;
}

impl<T: AsRef<[u8]>> Read4Opcode for Cursor<T> {
    fn u32(&mut self) -> Result<u32, io::Error> {
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        Ok(u32::from_be_bytes(bytes))
    }
    fn f64(&mut self) -> Result<f64, io::Error> {
        let mut bytes = [0u8; 8];
        self.read_exact(&mut bytes)?;
        Ok(f64::from_be_bytes(bytes))
    }
    fn shift<const N: usize>(&mut self) -> Result<(), io::Error> {
        let mut bytes = [0u8; N];
        self.read_exact(&mut bytes)?;
        Ok(())
    }
}
