use super::{CFAPattern, ColorMatrix, Parse, WhiteBalance};
use crate::{Error, ToReport};
use erreport::Report;
use std::io::{BufReader, Read, Seek};

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
            0x0111 thumbnail0
            0x0117 thumbnail_len0
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
            0x0111 thumbnail1
            0x0117 thumbnail_len1
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
            0x0111 thumbnail2
            0x0117 thumbnail_len2
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
            0x0111 thumbnail3
            0x0117 thumbnail_len3
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
    pub width: usize,
    pub height: usize,
    pub orientation: u16,
    pub compression: u16,
    pub cfa_pattern: CFAPattern,
    pub black_level: u16,
    pub scaleup_factor: u16,
    pub white_balance: WhiteBalance,
    pub thumbnail_addr: u64,
    pub thumbnail_size: usize,
    pub tile_offsets: Box<[u32]>,
    pub tile_byte_counts: Box<[u32]>,
    pub tile_width: u32,
    pub tile_len: u32,
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
        let orientation = get!(orientation -> u16);
        let white_balance = get!(white_balance => r64s);
        let white_balance = [
            (1024f64 / white_balance[0]) as u16,
            1024,
            (1024f64 / white_balance[2]) as u16,
        ];

        // ifd 0 is the main ifd
        let cfa_pattern0 = get!(cfa_pattern0);
        if cfa_pattern0.is_some() {
            let thumbnail_addr = get!(thumbnail0 -> u32) as u64;
            let thumbnail_size = get!(thumbnail_len0 -> u32) as usize;

            let width = get!(width0 -> u32) as usize;
            let height = get!(height0 -> u32) as usize;
            let compression = get!(compression0 -> u16);
            let tile_offsets = get!(tile_offsets0 => u32s);
            let tile_byte_counts = get!(tile_byte_counts0 => u32s);
            let tile_width = get!(tile_width0 -> u32);
            let tile_len = get!(tile_len0 -> u32);
            let white_level = get!(white_level0 -> u16);
            let black_level = get!(black_level0 -> u16);
            let cfa_pattern = get!(cfa_pattern0 -> raw);

            let scaleup_factor = match white_level {
                16383 => 2,
                _ => 1,
            };

            return Ok(DngInfo {
                is_le,
                width,
                height,
                orientation,
                cfa_pattern: cfa_pattern.into(),
                compression,
                black_level,
                scaleup_factor,
                white_balance: white_balance.into(),
                thumbnail_addr,
                thumbnail_size,
                tile_offsets,
                tile_byte_counts,
                tile_width,
                tile_len,
                color_matrix_1,
                color_matrix_2,
            });
        }

        // ifd 1 is the main ifd
        let cfa_pattern1 = get!(cfa_pattern1);
        if cfa_pattern1.is_some() {}

        panic!()
    }
}
