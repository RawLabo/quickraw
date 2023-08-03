use crate::{Error, ToReport};
use erreport::Report;
use std::io::{BufReader, Read, Seek};

use super::ColorMatrix;

mod dcp_rule {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0xc614 unique_model
            0xc622 color_matrix
        }
    );
}

pub struct DcpInfo {
    pub unique_model: Box<str>,
    pub color_matrix: ColorMatrix,
}

pub(crate) fn parse_exif<T: Read + Seek>(mut reader: T) -> Result<DcpInfo, Report> {
    let buf_reader = BufReader::new(&mut reader);
    let (exif, _) = quickexif::parse_exif(buf_reader, dcp_rule::PATH_LST, None).to_report()?;

    super::gen_get!(exif, dcp_rule);

    let unique_model = get!(unique_model => str);
    let color_matrix = get!(color_matrix => r64s);

    Ok(DcpInfo {
        unique_model: unique_model.into(),
        color_matrix: color_matrix.into(),
    })
}
