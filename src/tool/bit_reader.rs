pub(crate) struct BitReader<'a> {
    source: &'a [u8],
    position: usize,
    cache: u32,
    cached_bits: usize,
}

impl<'a> BitReader<'a> {
    pub(crate) fn new(source: &'a [u8]) -> Self {
        BitReader {
            source,
            position: 0,
            cache: 0,
            cached_bits: 0,
        }
    }

    /// bits must be greater than 0 and less than 32
    /// it will keep reading out 0 after it's finished
    #[inline(always)]
    pub(crate) fn check_bits_jpeg(&mut self, bits: usize) -> u32 {
        let position = self.position;
        let cache = self.cache;
        let cached_bits = self.cached_bits;

        let result = self.read_bits_jpeg(bits);
        self.position = position;
        self.cache = cache;
        self.cached_bits = cached_bits;

        result
    }

    /// bits must be greater than 0 and less than 32
    /// it will keep reading out 0 after it's finished
    #[inline(always)]
    pub(crate) fn read_bits_jpeg(&mut self, bits: usize) -> u32 {
        while self.cached_bits < bits {
            let byte = if let Some(byte) = self.source.get(self.position) {
                *byte
            } else {
                0
            };

            self.cache <<= 8;
            self.cache |= byte as u32;
            self.cached_bits += 8;

            if byte == 0xff {
                self.position += 2;
            } else {
                self.position += 1;
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

    /// bits must be greater than 0 and less than 32
    /// it will keep reading out 0 after it's finished
    pub(crate) fn read_bits_be(&mut self, bits: usize) -> u32 {
        while self.cached_bits < bits {
            let byte = if let Some(byte) = self.source.get(self.position) {
                *byte
            } else {
                0
            };

            self.cache <<= 8;
            self.cache |= byte as u32;
            self.cached_bits += 8;

            self.position += 1;
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

    /// bits must be greater than 0 and less than 32
    /// it will keep reading out 0 after it's finished
    pub(crate) fn read_bits_le(&mut self, bits: usize) -> u32 {
        while self.cached_bits < bits {
            let byte = if let Some(byte) = self.source.get(self.position) {
                *byte
            } else {
                0
            };

            self.cache |= (byte as u32) << self.cached_bits;
            self.cached_bits += 8;

            self.position += 1;
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
