use crate::ToReport;
use erreport::Report;
use std::io::{Read, Seek, SeekFrom};
use wide::i32x4;

pub(crate) mod arw;
pub(crate) mod base;
pub(crate) mod dcp;
pub(crate) mod dng;

/// These three traits represent three processes needed to decode: ParseExif -> Decode compressed bytes -> Preprocess of image
pub(crate) trait Parse<Info> {
    fn parse_exif<T: Read + Seek>(reader: T) -> Result<Info, Report>;
}
pub(crate) struct DecodingInfo {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) white_balance: WhiteBalance,
    pub(crate) cfa_pattern: Option<CFAPattern>,
    pub(crate) color_matrix: Option<ColorMatrix>,
}

#[derive(Copy, Clone)]
pub struct ColorMatrix {
    pub(crate) matrix: [f32; 9],
    pub(crate) column0: i32x4,
    pub(crate) column1: i32x4,
    pub(crate) column2: i32x4,
    pub(crate) clamp0: i32x4,
    pub(crate) clamp1: i32x4,
}
impl ColorMatrix {
    fn apply_analogbalance(&mut self, ab: &[f64]) {
        let a = [
            ab[0] as f32,
            0.,
            0.,
            0.,
            ab[1] as f32,
            0.,
            0.,
            0.,
            ab[2] as f32,
        ];
        let b = self.matrix;
        self.matrix = [
            a[0] * b[0] + a[1] * b[3] + a[2] * b[6],
            a[0] * b[1] + a[1] * b[4] + a[2] * b[7],
            a[0] * b[2] + a[1] * b[5] + a[2] * b[8],
            a[3] * b[0] + a[4] * b[3] + a[5] * b[6],
            a[3] * b[1] + a[4] * b[4] + a[5] * b[7],
            a[3] * b[2] + a[4] * b[5] + a[5] * b[8],
            a[6] * b[0] + a[7] * b[3] + a[8] * b[6],
            a[6] * b[1] + a[7] * b[4] + a[8] * b[7],
            a[6] * b[2] + a[7] * b[5] + a[8] * b[8],
        ];
    }
}
impl From<[f32; 9]> for ColorMatrix {
    fn from(value: [f32; 9]) -> Self {
        Self {
            matrix: value,
            column0: i32x4::ZERO,
            column1: i32x4::ZERO,
            column2: i32x4::ZERO,
            clamp0: i32x4::splat(0xffff),
            clamp1: i32x4::splat(0),
        }
    }
}
impl From<&[f32; 9]> for ColorMatrix {
    fn from(value: &[f32; 9]) -> Self {
        let mut matrix = [0f32; 9];
        matrix.copy_from_slice(value);
        matrix.into()
    }
}
impl From<Box<[f64]>> for ColorMatrix {
    fn from(value: Box<[f64]>) -> Self {
        let mut c = value.iter().map(|&x| x as f32).collect::<Box<_>>();
        matrix3_inverse(&mut c);
        matrix3_normalize(&mut c);
        let mut matrix = [0f32; 9];
        matrix.iter_mut().zip(c.iter()).for_each(|(dst, src)| {
            *dst = *src;
        });
        matrix.into()
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
    pub(crate) rgb: i32x4,
    pub(crate) bit_shift: i32,
    pub(crate) clamp: i32x4,
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
            rgb: i32x4::from([r as i32, g as i32, b as i32, 0]),
            bit_shift,
            clamp: i32x4::splat(0xffff),
        }
    }
}

pub enum CFAPattern {
    Rggb,
    Grbg,
    Gbrg,
    Bggr,
    XTrans0, // RBGBRG
    XTrans1, // GGRGGB
}
impl<'a> From<&'a [u8]> for CFAPattern {
    fn from(value: &'a [u8]) -> Self {
        match value {
            [0, 1, 1, 2] => CFAPattern::Rggb,
            [2, 1, 1, 0] => CFAPattern::Bggr,
            [1, 0, 2, 1] => CFAPattern::Grbg,
            [1, 2, 0, 1] => CFAPattern::Gbrg,
            _ => CFAPattern::Rggb,
        }
    }
}

pub(crate) fn get_bytes<T: Read + Seek>(
    mut reader: T,
    addr: u64,
    size: usize,
) -> Result<Box<[u8]>, Report> {
    let mut bytes = vec![0u8; size];
    reader.seek(SeekFrom::Start(addr)).to_report()?;
    reader.read_exact(&mut bytes).to_report()?;

    Ok(bytes.into_boxed_slice())
}

macro_rules! gen_get {
    ($exif:expr, $rule:tt) => {
        macro_rules! get {
            ($tag:tt) => {
                $exif.get($rule::$tag)
            };
            ($tag:expr) => {
                $exif.get($tag)
            };
            ($tag:tt => $fn:tt) => {
                $exif
                    .get($rule::$tag)
                    .and_then(|x| x.$fn())
                    .ok_or(Error::IsNone)
                    .to_report()?
            };
            ($tag:expr => $fn:tt) => {
                $exif
                    .get($tag)
                    .and_then(|x| x.$fn())
                    .ok_or(Error::IsNone)
                    .to_report()?
            };
            ($tag:tt, $fn:tt) => {
                $exif
                    .get($rule::$tag)
                    .map(|x| x.$fn())
                    .ok_or(Error::IsNone)
                    .to_report()?
            };
            ($tag:expr, $fn:tt) => {
                $exif
                    .get($tag)
                    .map(|x| x.$fn())
                    .ok_or(Error::IsNone)
                    .to_report()?
            };
        }
    };
}
pub(self) use gen_get;

fn get_scaleup_factor(white_level: u16) -> u16 {
    let mut factor = 0;
    let mut v = white_level as u32;
    let max = u16::MAX as u32;
    while (v << 1) <= max {
        v <<= 1;
        factor += 1;
    }
    factor
}
