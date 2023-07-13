pub(crate) struct BitReader<'a, T: Iterator<Item = &'a u8>> {
    source: T,
    cache: u32,
    cached_bits: usize,
}
impl<'a, T: Iterator<Item = &'a u8>> BitReader<'a, T> {
    fn read_bits_be(&mut self, bits: usize) -> u32 {
        while self.cached_bits < bits {
            match self.source.next() {
                Some(byte) => {
                    self.cache <<= 8;
                    self.cache |= *byte as u32;
                    self.cached_bits += 8;
                }
                None => panic!("bits:{bits} too much"),
            }
        }

        let preserved_bits = self.cached_bits - bits;
        let preserved_mask = (1u32 << preserved_bits) - 1;
        let preserved_cache = self.cache & preserved_mask;

        let mask = (1u32 << bits) - 1;
        let result = (self.cache >> preserved_bits) & mask;

        self.cache = preserved_cache;
        self.cached_bits = preserved_bits;
        result
    }
    fn read_bits_le(&mut self, bits: usize) -> u32 {
        while self.cached_bits < bits {
            match self.source.next() {
                Some(byte) => {
                    self.cache |= (*byte as u32) << self.cached_bits;
                    self.cached_bits += 8;
                }
                None => panic!("bits:{bits} too much"),
            }
        }

        let preserved_bits = self.cached_bits - bits;
        let preserved_cache = self.cache >> bits;

        let mask = (1u32 << bits) - 1;
        let result = self.cache & mask;

        self.cache = preserved_cache;
        self.cached_bits = preserved_bits;
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::tool::bit_reader::BitReader;

    enum Bits {
        U8(u8),
        U16(u16),
        U32(u32),
    }
    impl Bits {
        fn as_u32(self) -> u32 {
            match self {
                Self::U8(x) => x as u32,
                Self::U16(x) => x as u32,
                Self::U32(x) => x,
            }
        }
    }

    fn bit_read_be(data: &[u8], nth_bits: usize, size: usize) -> Bits {
        let mask = (1u32 << size) - 1;
        let byte_offset = nth_bits / 8;
        let bit_index = nth_bits % 8;

        let range = bit_index + size;

        if range <= 8 {
            let mut bytes = [0u8; 1];
            bytes.copy_from_slice(&data[byte_offset..byte_offset + 1]);
            Bits::U8((u8::from_be_bytes(bytes) >> (8 - bit_index - size)) & mask as u8)
        } else if range <= 16 {
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&data[byte_offset..byte_offset + 2]);
            Bits::U16((u16::from_be_bytes(bytes) >> (16 - bit_index - size)) & mask as u16)
        } else {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[byte_offset..byte_offset + 4]);
            Bits::U32((u32::from_be_bytes(bytes) >> (32 - bit_index - size)) & mask)
        }
    }

    fn bit_read_le(data: &[u8], nth_bits: usize, size: usize) -> Bits {
        let mask = (1u32 << size) - 1;
        let byte_offset = nth_bits / 8;
        let bit_index = nth_bits % 8;

        let range = bit_index + size;

        if range <= 8 {
            let mut bytes = [0u8; 1];
            bytes.copy_from_slice(&data[byte_offset..byte_offset + 1]);
            Bits::U8((u8::from_le_bytes(bytes) >> bit_index) & mask as u8)
        } else if range <= 16 {
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&data[byte_offset..byte_offset + 2]);
            Bits::U16((u16::from_le_bytes(bytes) >> bit_index) & mask as u16)
        } else {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[byte_offset..byte_offset + 4]);
            Bits::U32((u32::from_le_bytes(bytes) >> bit_index) & mask)
        }
    }

    #[test]
    fn test_le() {
        let data = vec![
            0b10001101u8,
            0b11010100,
            0b10011001,
            0b11110001,
            0b00001011,
            0b10011011,
        ];

        {
            let mut reader = BitReader {
                source: data.iter(),
                cache: 0,
                cached_bits: 0,
            };
            reader.read_bits_le(11);
            let r0 = reader.read_bits_le(7);

            let r1 = bit_read_le(&data, 11, 7);

            assert_eq!(r0, r1.as_u32());
        }

        
        {
            let mut reader = BitReader {
                source: data.iter(),
                cache: 0,
                cached_bits: 0,
            };
            reader.read_bits_be(11);
            let r0 = reader.read_bits_be(7);

            let r1 = bit_read_be(&data, 11, 7);

            assert_eq!(r0, r1.as_u32());
        }
    }
}
