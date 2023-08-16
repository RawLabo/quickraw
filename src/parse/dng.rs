use super::{CFAPattern, ColorMatrix, Parse, WhiteBalance};
use crate::{Error, ToReport, parse::get_scaleup_factor};
use erreport::Report;
use std::io::{BufReader, Read, Seek};

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
            0xc621 color_matrix_1  // for apple pro raw
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
        0 -> 0x014a -> 200 {
            0x0100 width3
            0x0101 height3
            0x0102 bps3
            0x0103 compression3
            0x0111 strip_addr3
            0x0117 strip_size3
            0x828e cfa_pattern3
            0x0144 tile_offsets3
            0x0145 tile_byte_counts3
            0x0142 tile_width3
            0x0143 tile_len3
            0xc61d white_level3
            0xc61a black_level3
            0xc61f crop_xy3
            0xc620 crop_size3
        }
    );
}

pub struct DngInfo {
    pub is_le: bool,
    pub is_converted: bool,
    pub compression: u16,
    pub strip_addr: u64,
    pub strip_size: usize,
    pub tile_offsets: Box<[u32]>,
    pub tile_byte_counts: Box<[u32]>,
    pub tile_width: u32,
    pub tile_len: u32,

    pub width: usize,
    pub height: usize,
    pub orientation: u16,
    pub cfa_pattern: CFAPattern,
    pub black_level: u16,
    pub scaleup_factor: u16,
    pub white_balance: WhiteBalance,
    pub thumbnail: Option<(u64, usize)>,
    pub color_matrix_1: ColorMatrix,
    pub color_matrix_2: ColorMatrix,
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

        let cfa_pattern0 = get!(cfa_pattern0);
        let cfa_pattern1 = get!(cfa_pattern1);
        let cfa_pattern2 = get!(cfa_pattern2);
        let tags: [&(u16, u16); 14] = if cfa_pattern0.is_some() {
            [
                dng_rule::compression0,
                // either strip or tile
                dng_rule::strip_addr0,
                dng_rule::strip_size0,
                dng_rule::tile_offsets0,
                dng_rule::tile_byte_counts0,
                dng_rule::tile_width0,
                dng_rule::tile_len0,
                // for thumbnail
                dng_rule::strip_addr1,
                dng_rule::strip_size1,
                // other info
                dng_rule::width0,
                dng_rule::height0,
                dng_rule::white_level0,
                dng_rule::black_level0,
                dng_rule::cfa_pattern0,
            ]
        } else if cfa_pattern1.is_some() {
            [
                dng_rule::compression1,
                // either strip or tile
                dng_rule::strip_addr1,
                dng_rule::strip_size1,
                dng_rule::tile_offsets1,
                dng_rule::tile_byte_counts1,
                dng_rule::tile_width1,
                dng_rule::tile_len1,
                // for thumbnail
                dng_rule::strip_addr2,
                dng_rule::strip_size2,
                // other info
                dng_rule::width1,
                dng_rule::height1,
                dng_rule::white_level1,
                dng_rule::black_level1,
                dng_rule::cfa_pattern1,
            ]
        } else if cfa_pattern2.is_some() {
            [
                dng_rule::compression2,
                // either strip or tile
                dng_rule::strip_addr2,
                dng_rule::strip_size2,
                dng_rule::tile_offsets2,
                dng_rule::tile_byte_counts2,
                dng_rule::tile_width2,
                dng_rule::tile_len2,
                // for thumbnail
                dng_rule::strip_addr3,
                dng_rule::strip_size3,
                // other info
                dng_rule::width2,
                dng_rule::height2,
                dng_rule::white_level2,
                dng_rule::black_level2,
                dng_rule::cfa_pattern2,
            ]
        } else {
            return Err(DngError::NoCfaPatternFound).to_report();
        };

        let compression = get!(tags[0], u16);

        let (strip_addr, strip_size, tile_offsets, tile_byte_counts, tile_width, tile_len) =
            match compression {
                1 => {
                    let strip_addr = get!(tags[1], u32) as u64;
                    let strip_size = get!(tags[2], u32) as usize;
                    (strip_addr, strip_size, [].into(), [].into(), 0, 0)
                }
                7 => {
                    let tile_offsets = get!(tags[3] => u32s);
                    let tile_byte_counts = get!(tags[4] => u32s);
                    let tile_width = get!(tags[5], u32);
                    let tile_len = get!(tags[6], u32);
                    (0, 0, tile_offsets, tile_byte_counts, tile_width, tile_len)
                }
                _ => return Err(DngError::CompressionTypeNotSupported(compression)).to_report(),
            };

        let thumbnail = {
            if let (Some(thumb_addr), Some(thumb_size)) = (get!(tags[7]), get!(tags[8])) {
                Some((thumb_addr.u32() as u64, thumb_size.u32() as usize))
            } else {
                None
            }
        };

        let width = get!(tags[9], u32) as usize;
        let height = get!(tags[10], u32) as usize;
        let white_level = get!(tags[11], u16);
        let black_level = if let Some(bl) = exif.get(tags[12]).and_then(|x| x.r64s()) {
            bl[0] as u16
        } else {
            get!(tags[12], u16)
        };
        let cfa_pattern = get!(tags[13], raw);

        let scaleup_factor = get_scaleup_factor(white_level);
        
        Ok(DngInfo {
            is_le,
            is_converted,
            width,
            height,
            orientation,
            cfa_pattern: cfa_pattern.into(),
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
        })
    }
}
