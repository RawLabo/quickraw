use crate::report::{Report, ToReport};
use std::io::{Read, Seek, SeekFrom};

pub(crate) mod arw;
pub(crate) mod dcp;

pub struct ColorMatrix {
    pub(crate) matrix: [f32; 9],
    pub(crate) matrix_with_colorspace: [i32; 9],
}
impl From<&[f32; 9]> for ColorMatrix {
    fn from(value: &[f32; 9]) -> Self {
        let mut matrix = [0f32; 9];
        matrix.copy_from_slice(value);
        Self {
            matrix,
            matrix_with_colorspace: [1, 0, 0, 0, 1, 0, 0, 0, 1], // use identical matrix by default
        }
    }
}
impl From<Box<[f64]>> for ColorMatrix {
    fn from(value: Box<[f64]>) -> Self {
        let mut c = value.into_iter().map(|&x| x as f32).collect::<Vec<_>>();
        matrix3_inverse(&mut c);
        matrix3_normalize(&mut c);
        let mut matrix = [0f32; 9];
        matrix.iter_mut().zip(c.into_iter()).for_each(|(dst, src)| {
            *dst = src;
        });
        Self {
            matrix,
            matrix_with_colorspace: [1, 0, 0, 0, 1, 0, 0, 0, 1],
        }
    }
}

fn matrix3_normalize(x: &mut [f32]) {
    assert!(x.len() == 9);
    x.chunks_exact_mut(3).for_each(|x| {
        let sum = x.iter().sum::<f32>();
        x.iter_mut().for_each(|x| *x /= sum);
    });
}
fn matrix3_inverse(x: &mut [f32]) {
    assert!(x.len() == 9);
    let m11 = x[0];
    let m12 = x[3];
    let m13 = x[6];

    let m21 = x[1];
    let m22 = x[4];
    let m23 = x[7];

    let m31 = x[2];
    let m32 = x[5];
    let m33 = x[8];

    let minor_m12_m23 = m22 * m33 - m32 * m23;
    let minor_m11_m23 = m21 * m33 - m31 * m23;
    let minor_m11_m22 = m21 * m32 - m31 * m22;

    let determinant = m11 * minor_m12_m23 - m12 * minor_m11_m23 + m13 * minor_m11_m22;

    x[0] = minor_m12_m23 / determinant;
    x[1] = (m13 * m32 - m33 * m12) / determinant;
    x[2] = (m12 * m23 - m22 * m13) / determinant;

    x[3] = -minor_m11_m23 / determinant;
    x[4] = (m11 * m33 - m31 * m13) / determinant;
    x[5] = (m13 * m21 - m23 * m11) / determinant;

    x[6] = minor_m11_m22 / determinant;
    x[7] = (m12 * m31 - m32 * m11) / determinant;
    x[8] = (m11 * m22 - m21 * m12) / determinant;
}

pub struct WhiteBalance {
    pub(crate) r: i32,
    pub(crate) g: i32,
    pub(crate) b: i32,
    pub(crate) bit_shift: i32,
}
impl From<[u16; 3]> for WhiteBalance {
    fn from([r, g, b]: [u16; 3]) -> Self {
        let mut bit_shift = 0;
        for i in 1.. {
            if (g >> i) == 1 {
                bit_shift = i;
                break;
            }
        }

        Self {
            r: r as i32,
            g: g as i32,
            b: b as i32,
            bit_shift,
        }
    }
}

pub enum CFAPattern {
    RGGB,
    GRBG,
    GBRG,
    BGGR,
    XTrans0, // RBGBRG
    XTrans1, // GGRGGB
}
impl<'a> From<&'a [u8]> for CFAPattern {
    fn from(value: &'a [u8]) -> Self {
        match value {
            [0, 1, 1, 2] => CFAPattern::RGGB,
            [2, 1, 1, 0] => CFAPattern::BGGR,
            [1, 0, 2, 1] => CFAPattern::GRBG,
            [1, 2, 0, 1] => CFAPattern::GBRG,
            _ => CFAPattern::RGGB,
        }
    }
}

pub(crate) fn get_bytes<T: Read + Seek>(
    mut reader: T,
    addr: u64,
    size: usize,
) -> Result<Vec<u8>, Report> {
    let mut bytes = vec![0u8; size];
    reader.seek(SeekFrom::Start(addr)).to_report()?;
    reader.read_exact(&mut bytes).to_report()?;

    Ok(bytes)
}

macro_rules! gen_get {
    ($exif:expr, $rule:tt) => {
        macro_rules! get {
            ($tag:tt => $fn:tt) => {
                $exif.get($rule::$tag)
                    .and_then(|x| x.$fn())
                    .ok_or(Error::IsNone)
                    .to_report()?
            };
            ($tag:tt -> $fn:tt) => {
                $exif.get($rule::$tag)
                    .map(|x| x.$fn())
                    .ok_or(Error::IsNone)
                    .to_report()?
            };
        }
    };
}
pub(crate) use gen_get;