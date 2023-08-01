use std::io::{BufReader, Read, Seek};

use crate::{
    decode,
    report::{Report, ToReport},
    tool::tone_curve::gen_tone_curve_sony,
    Error,
};

use super::{CFAPattern, WhiteBalance};

mod arw_rule {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 -> 0xc634 -> 0 {}
        0 -> 0xc634 -> 0 -> 0x7200 -> 0xffff {
            0x7310 black_level
            0x7312 white_balance
            0x787f white_level
        }
        0 {
            0x010f make
            0x0110 model
            0x0112 orientation
            0x0201 preview_offset
            0x0202 preview_len
        }
        0 -> 0x8769 -> 0 {
            0x9102 compressed_bps
        }
        0 -> 0x014a -> 0 {
            0x0103 compression
            0x0100 width
            0x0101 height
            0x0102 bps
            0x828e cfa_pattern
            0x0111 strip
            0x0117 strip_len
            0x7010 tone_curve
            0xc61f crop_xy
            0xc620 crop_wh
        }
    );
}

pub struct ArwInfo {
    pub is_le: bool,
    pub width: usize,
    pub height: usize,
    pub orientation: u16,
    pub compression: u16,
    pub cfa_pattern: CFAPattern,
    pub black_level: u16,
    pub scaleup_factor: u16,
    pub white_balance: WhiteBalance,
    pub tone_curve: [u16; 4096],
    pub strip_addr: u64,
    pub strip_size: usize,
    pub thumbnail_addr: u64,
    pub thumbnail_size: usize,
}

impl decode::Parse<ArwInfo> for ArwInfo {
    fn get_strip_info(&self) -> (u64, usize) {
        (self.strip_addr, self.strip_size)
    }
    fn parse_exif<T: Read + Seek>(mut reader: T) -> Result<ArwInfo, Report> {
        let buf_reader = BufReader::new(&mut reader);
        let (exif, is_le) =
            quickexif::parse_exif(buf_reader, arw_rule::PATH_LST, Some((0, 1))).to_report()?;

        super::gen_get!(exif, arw_rule);

        let width = get!(width -> u16);
        let height = get!(height -> u16);
        let compression = get!(compression -> u16);
        let orientation = get!(orientation -> u16);
        let black_level = get!(black_level => u16s);
        let white_balance = get!(white_balance => u16s);
        let cfa_pattern = get!(cfa_pattern -> raw);

        let image_addr = get!(strip -> u32) as u64;
        let image_size = get!(strip_len -> u32) as usize;

        let thumbnail_addr = get!(preview_offset -> u32) as u64;
        let thumbnail_size = get!(preview_len -> u32) as usize;

        let tone_curve_points = get!(tone_curve => u16s);
        let tone_curve = gen_tone_curve_sony(&tone_curve_points);

        let white_level =
            if let Some(values) = exif.get(arw_rule::white_level).and_then(|x| x.u16s()) {
                values[0]
            } else {
                get!(white_level -> u16)
            };
        let scaleup_factor = match white_level {
            15360 => 2,
            _ => 1,
        };

        Ok(ArwInfo {
            is_le,
            compression,
            black_level: black_level[0],
            white_balance: [white_balance[0], white_balance[1], white_balance[3]].into(),
            scaleup_factor,
            cfa_pattern: cfa_pattern.into(),
            tone_curve,
            width: width as usize,
            height: height as usize,
            orientation,
            strip_addr: image_addr,
            strip_size: image_size,
            thumbnail_addr,
            thumbnail_size,
        })
    }
}
