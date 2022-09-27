pub(super) trait GetNumFromBytes {
    fn u16(&self, is_le: bool, start: usize) -> u16;
    fn u16le(&self, start: usize) -> u16;
    fn u16be(&self, start: usize) -> u16;
    fn u32(&self, is_le: bool, start: usize) -> u32;
    fn u32le(&self, start: usize) -> u32;
    fn u32be(&self, start: usize) -> u32;
    fn i32(&self, is_le: bool, start: usize) -> i32;
    fn r64(&self, is_le: bool, start: usize) -> f64;
}

pub(super) trait GetBytesFromInt<T> {
    fn to_bytes(self, is_le: bool) -> T;
}

macro_rules! gen_impl_get_int {
    ($t:tt, $len:expr) => {
        fn $t(&self, is_le: bool, start: usize) -> $t {
            let bytes: [u8; $len] = *&self[start..start + $len].try_into().unwrap();
            if is_le {
                $t::from_le_bytes(bytes)
            } else {
                $t::from_be_bytes(bytes)
            }
        }
    };
}
impl GetNumFromBytes for &[u8] {
    gen_impl_get_int!(u16, 2);
    gen_impl_get_int!(u32, 4);
    gen_impl_get_int!(i32, 4);

    fn u16le(&self, start: usize) -> u16 {
        let bytes: [u8; 2] = self[start..start + 2].try_into().unwrap();
        u16::from_le_bytes(bytes)
    }
    fn u16be(&self, start: usize) -> u16 {
        let bytes: [u8; 2] = self[start..start + 2].try_into().unwrap();
        u16::from_be_bytes(bytes)
    }
    fn u32le(&self, start: usize) -> u32 {
        let bytes: [u8; 4] = self[start..start + 4].try_into().unwrap();
        u32::from_le_bytes(bytes)
    }
    fn u32be(&self, start: usize) -> u32 {
        let bytes: [u8; 4] = self[start..start + 4].try_into().unwrap();
        u32::from_be_bytes(bytes)
    }

    fn r64(&self, is_le: bool, start: usize) -> f64 {
        let n = self.i32(is_le, start) as f64;
        let d = self.u32(is_le, start + 4) as f64;
        n / d
    }
}

macro_rules! gen_get_bytes_impls {
    ($t:ty, $n:expr) => {
        impl GetBytesFromInt<[u8; $n]> for $t {
            fn to_bytes(self, is_le: bool) -> [u8; $n] {
                if is_le {
                    self.to_le_bytes()
                } else {
                    self.to_be_bytes()
                }
            }
        }
    };
}
gen_get_bytes_impls!(u16, 2);
gen_get_bytes_impls!(u32, 4);


#[inline(always)]
pub(super) fn to_16bit_iter(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(2).map(move |bytes| bytes.u16(is_le, 0))
}
#[inline(always)]
pub(super) fn to_14bit_iter(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(2).map(move |bytes| bytes.u16(is_le, 0) & 0x3fff)
}
#[inline(always)]
pub(super) fn to_14bit_iter_packed(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
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
pub(super) fn to_12bit_iter(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(2).map(move |bytes| bytes.u16(is_le, 0) & 0x0fff)
}
#[inline(always)]
pub(super) fn _to_12bit_left_aligned_iter(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
    buffer.chunks_exact(2).map(move |bytes| bytes.u16(is_le, 0) >> 4)
}
#[inline(always)]
pub(super) fn to_12bit_iter_packed(buffer: &[u8], is_le: bool) -> impl Iterator<Item = u16> + '_ {
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


pub(super) fn matrix3_normalize(x: &mut [f32]) {
    assert!(x.len() == 9);
    x.chunks_exact_mut(3).for_each(|x| {
        let sum = x.iter().sum::<f32>();
        x.iter_mut().for_each(|x| *x /= sum);
    });
}

pub(super) fn matrix3_inverse(x: &mut [f32]) {
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
