use crate::tiff::utility::GetNumFromBytes;

use super::*;

#[inline(always)]
pub fn to_16bit_iter(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(2).map(move |bytes| bytes.u16(is_le, 0))
}
#[inline(always)]
pub fn to_14bit_iter(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(2).map(move |bytes| bytes.u16(is_le, 0) & 0x3fff)
}
#[inline(always)]
pub fn to_14bit_iter_packed(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(7).flat_map(move |bytes| {
        let g1 = bytes[0] as u16;
        let g2 = bytes[1] as u16;
        let g3 = bytes[2] as u16;
        let g4 = bytes[3] as u16;
        let g5 = bytes[4] as u16;
        let g6 = bytes[5] as u16;
        let g7 = bytes[6] as u16;

        // 11111111 11111111 11111111 11111111 11111111 11111111 11111111
        // aaaaaaaa aaaaaabb bbbbbbbb bbbbcccc cccccccc ccdddddd dddddddd
        // aaaaaaaa bbaaaaaa bbbbbbbb ccccbbbb cccccccc ddddddcc dddddddd
        if is_le {
            [
                ((g2 & 0b111111) << 8) | g1,
                ((g4 & 0x0f) << 10) | (g3 << 2) | (g2 >> 6),
                ((g6 & 0b11) << 12) | (g5 << 4) | (g4 >> 4),
                (g7 << 6) | (g6 >> 2),
            ]
        } else {
            [
                (g1 << 6) | (g2 >> 2),
                ((g2 & 0b11) << 12) | (g3 << 4) | (g4 >> 4),
                ((g4 & 0b1111) << 10) | (g5 << 2) | (g6 >> 6),
                ((g6 & 0b111111) << 8) | g7,
            ]
        }
    })
}

#[inline(always)]
pub fn to_12bit_iter(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(2).map(move |bytes| bytes.u16(is_le, 0) & 0x0fff)
}
#[inline(always)]
pub fn to_12bit_left_aligned_iter(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(2).map(move |bytes| bytes.u16(is_le, 0) >> 4)
}
#[inline(always)]
pub fn to_12bit_iter_packed(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(3).flat_map(move |bytes| {
        let g1 = bytes[0] as u16;
        let g2 = bytes[1] as u16;
        let g3 = bytes[2] as u16;

        if is_le {
            [((g2 & 0x0f) << 8) | g1, (g3 << 4) | (g2 >> 4)]
        } else {
            [(g1 << 4) | (g2 >> 4), ((g2 & 0x0f) << 8) | g3]
        }
    })
}

pub fn basic_info_task() -> ExifTask {
    create_rule![
        tiff {
            0x010f {
                str + 0 / make
            }
            0x0110 {
                str + 0 / model
            }
            0xc612? / dng_version
            if dng_version ? {
                0xc614 {
                    str + 0 / make_model
                }
                0xc622 {
                    r64 + 0 / c0
                    r64 + 1 / c1
                    r64 + 2 / c2
                    r64 + 3 / c3
                    r64 + 4 / c4
                    r64 + 5 / c5
                    r64 + 6 / c6
                    r64 + 7 / c7
                    r64 + 8 / c8
                }
            }
        }
    ]
}

pub fn matrix3_normalize(x: &mut [f32]) {
    assert!(x.len() == 9);
    x.chunks_exact_mut(3).for_each(|x| {
        let sum = x.iter().sum::<f32>();
        x.iter_mut().for_each(|x| *x /= sum);
    });
}

pub fn matrix3_inverse(x: &mut [f32]) {
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
