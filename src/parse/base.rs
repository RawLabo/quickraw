use std::io::{BufReader, Read, Seek, SeekFrom};

use crate::{
    report::{Report, ToReport},
    Error,
};

mod base_rule {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;
    gen_tags_info!(
        0 {
            0x010f make
            0x0110 model
        }
    );
}

pub(crate) enum Kind {
    Arw,
    Cr2,
    Cr3,
    Dng,
    Nef,
    Orf,
    Raf,
    Rw2,
    Unsupported,
}


pub(crate) fn detect<T: Read + Seek>(mut reader: T) -> Result<(Kind, Box<str>), Report> {
    let mut buf_reader = BufReader::new(&mut reader);

    let mut header = [0u8; 16];
    buf_reader.read_exact(&mut header).to_report()?;
    buf_reader.seek_relative(-16).to_report()?;

    let kind = match &header[..2] {
        [0x49, 0x49] | [0x4d, 0x4d] => None,
        _ => {
            if &header[..4] == [0x46, 0x55, 0x4a, 0x49] {
                quickexif::seek_header_raf(&mut buf_reader, 0).to_report()?;
                Some(Kind::Raf)
            } else if &header
                == &[
                    0, 0, 0, 0x18, 0x66, 0x74, 0x79, 0x70, 0x63, 0x72, 0x78, 0x20, 0, 0, 0, 1,
                ]
            {
                quickexif::seek_header_cr3(&mut buf_reader, 0).to_report()?;
                Some(Kind::Cr3)
            } else {
                return Err(Error::UnsupportedRawFile).to_report();
            }
        }
    };

    let (exif, _) = quickexif::parse_exif(buf_reader, base_rule::PATH_LST, None).to_report()?;
    // recover cursor position for later parsing
    reader.seek(SeekFrom::Start(0)).to_report()?;

    let make = exif
        .get(base_rule::make)
        .and_then(|x| x.str())
        .ok_or(Error::IsNone)
        .to_report()?;
    let model = exif
        .get(base_rule::model)
        .and_then(|x| x.str())
        .ok_or(Error::IsNone)
        .to_report()?;

    let kind = kind.unwrap_or(match make {
        "SONY" => Kind::Arw,
        _ => Kind::Unsupported,
    });
    Ok((kind, model.into()))
}
