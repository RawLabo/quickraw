use super::bit_reader::BitReader;

#[derive(Debug)]
pub(crate) struct HuffmanDecoder {
    /// the index of the last non-zero value in huff_size
    max_bits: usize,
    /// [(symbol, bits),]
    lut: Box<[(u8, u8)]>,
}

impl HuffmanDecoder {
    pub(crate) fn read_next(&self, bit_reader: &mut BitReader) -> i32 {
        let v = bit_reader.check_bits_be(self.max_bits, true);
        let (symbol, bits) = self.lut[v as usize];
        bit_reader.read_bits_be(bits as usize, true);

        if symbol == 0 {
            return 0;
        }

        let mut diff = bit_reader.read_bits_be(symbol as usize, true) as i32;
        if diff >> (symbol - 1) == 0 {
            // is in the left negative range port of SSSS
            diff -= (1 << symbol) - 1;
        }

        diff
    }
    pub(crate) fn from_dht(dht: &quickexif::jpeg::DHT) -> Self {
        let mut max_bits = 16;
        loop {
            max_bits -= 1;
            match dht.huff_size.get(max_bits) {
                Some(0) => continue,
                Some(_) | None => break,
            }
        }
        max_bits += 1;

        let mut lut = vec![(0, 0); 1 << max_bits];
        let mut index = 0;
        for (len, vals) in dht.huff_vals.iter().enumerate() {
            for val in vals.iter() {
                for _ in 0..(1 << (max_bits - len - 1)) {
                    lut[index] = (*val, 1 + len as u8);
                    index += 1;
                }
            }
        }

        HuffmanDecoder {
            max_bits,
            lut: lut.into_boxed_slice(),
        }
    }
}
