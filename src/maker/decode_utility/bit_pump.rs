use super::super::utility::GetNumFromBytes;

#[derive(Debug, Copy, Clone)]
pub(in super::super) struct BitPumpMSB<'a> {
    buffer: &'a [u8],
    pos: usize,
    bits: u64,
    nbits: u32,
}

impl<'a> BitPumpMSB<'a> {
    pub(in super::super) fn new(src: &'a [u8]) -> BitPumpMSB {
        BitPumpMSB {
            buffer: src,
            pos: 0,
            bits: 0,
            nbits: 0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(in super::super) struct BitPumpMSB32<'a> {
    buffer: &'a [u8],
    pos: usize,
    bits: u64,
    nbits: u32,
}

impl<'a> BitPumpMSB32<'a> {
    pub(in super::super) fn new(src: &'a [u8]) -> BitPumpMSB32 {
        BitPumpMSB32 {
            buffer: src,
            pos: 0,
            bits: 0,
            nbits: 0,
        }
    }

    // #[inline(always)]
    // pub(in super::super) fn get_pos(&self) -> usize {
    //     self.pos - ((self.nbits >> 3) as usize)
    // }
}

impl<'a> BitPump for BitPumpMSB32<'a> {
    #[inline(always)]
    fn peek_bits(&mut self, num: u32) -> u32 {
        if num > self.nbits {
            let inbits: u64 = self.buffer.u32le(self.pos) as u64;
            self.bits = (self.bits << 32) | inbits;
            self.pos += 4;
            self.nbits += 32;
        }
        (self.bits >> (self.nbits - num)) as u32
    }

    #[inline(always)]
    fn consume_bits(&mut self, num: u32) {
        self.nbits -= num;
        self.bits &= (1 << self.nbits) - 1;
    }
}
#[derive(Debug, Copy, Clone)]
pub(in super::super) struct BitPumpJPEG<'a> {
    buffer: &'a [u8],
    pos: usize,
    bits: u64,
    nbits: u32,
    finished: bool,
}

impl<'a> BitPumpJPEG<'a> {
    pub(in super::super) fn new(src: &'a [u8]) -> BitPumpJPEG {
        BitPumpJPEG {
            buffer: src,
            pos: 0,
            bits: 0,
            nbits: 0,
            finished: false,
        }
    }
}

impl<'a> BitPump for BitPumpJPEG<'a> {
    #[inline(always)]
    fn peek_bits(&mut self, num: u32) -> u32 {
        if num > self.nbits && !self.finished {
            if self.pos < self.buffer.len() - 4
                && self.buffer[self.pos] != 0xff
                && self.buffer[self.pos + 1] != 0xff
                && self.buffer[self.pos + 2] != 0xff
                && self.buffer[self.pos + 3] != 0xff
            {
                let inbits: u64 = self.buffer.u32be(self.pos) as u64;
                self.bits = (self.bits << 32) | inbits;
                self.pos += 4;
                self.nbits += 32;
            } else {
                // Read 32 bits the hard way
                let mut read_bytes = 0;
                while read_bytes < 4 && !self.finished {
                    let byte = {
                        if self.pos >= self.buffer.len() {
                            self.finished = true;
                            0
                        } else {
                            let nextbyte = self.buffer[self.pos];
                            if nextbyte != 0xff {
                                nextbyte
                            } else if self.buffer[self.pos + 1] == 0x00 {
                                self.pos += 1; // Skip the extra byte used to mark 255
                                nextbyte
                            } else {
                                self.finished = true;
                                0
                            }
                        }
                    };
                    self.bits = (self.bits << 8) | (byte as u64);
                    self.pos += 1;
                    self.nbits += 8;
                    read_bytes += 1;
                }
            }
        }
        if num > self.nbits && self.finished {
            // Stuff with zeroes to not fail to read
            self.bits <<= 32;
            self.nbits += 32;
        }

        (self.bits >> (self.nbits - num)) as u32
    }

    #[inline(always)]
    fn consume_bits(&mut self, num: u32) {
        self.nbits -= num;
        self.bits &= (1 << self.nbits) - 1;
    }
}
#[derive(Debug, Copy, Clone)]
pub(in super::super) struct BitPumpLSB<'a> {
    buffer: &'a [u8],
    pos: usize,
    bits: u64,
    nbits: u32,
}

impl<'a> BitPumpLSB<'a> {
    pub(in super::super) fn new(src: &'a [u8]) -> BitPumpLSB {
        BitPumpLSB {
            buffer: src,
            pos: 0,
            bits: 0,
            nbits: 0,
        }
    }
}

pub(in super::super) struct BitPumpPanasonic<'a> {
    buffer: &'a [u8],
    pos: usize,
    nbits: u32,
    split: bool,
}

impl<'a> BitPumpPanasonic<'a> {
    pub(in super::super) fn new(src: &'a [u8], split: bool) -> BitPumpPanasonic {
        BitPumpPanasonic {
            buffer: src,
            pos: 0,
            nbits: 0,
            split,
        }
    }
}

pub(in super::super) trait BitPump {
    fn peek_bits(&mut self, num: u32) -> u32;
    fn consume_bits(&mut self, num: u32);

    #[inline(always)]
    fn get_bits(&mut self, num: u32) -> u32 {
        if num == 0 {
            return 0;
        }

        let val = self.peek_bits(num);
        self.consume_bits(num);

        val
    }

    #[inline(always)]
    fn peek_ibits(&mut self, num: u32) -> i32 {
        self.peek_bits(num) as i32
    }

    #[inline(always)]
    fn get_ibits(&mut self, num: u32) -> i32 {
        self.get_bits(num) as i32
    }

    // Sign extend ibits
    #[inline(always)]
    fn get_ibits_sextended(&mut self, num: u32) -> i32 {
        let val = self.get_ibits(num);
        val.wrapping_shl(32 - num).wrapping_shr(32 - num)
    }
}

impl<'a> BitPump for BitPumpLSB<'a> {
    #[inline(always)]
    fn peek_bits(&mut self, num: u32) -> u32 {
        if num > self.nbits {
            let inbits: u64 = self.buffer.u32le(self.pos) as u64;
            self.bits = ((inbits << 32) | (self.bits << (32 - self.nbits))) >> (32 - self.nbits);
            self.pos += 4;
            self.nbits += 32;
        }
        (self.bits & (0x0ffffffffu64 >> (32 - num))) as u32
    }

    #[inline(always)]
    fn consume_bits(&mut self, num: u32) {
        self.nbits -= num;
        self.bits >>= num;
    }
}

impl<'a> BitPump for BitPumpMSB<'a> {
    #[inline(always)]
    fn peek_bits(&mut self, num: u32) -> u32 {
        if num > self.nbits {
            let inbits: u64 = self.buffer.u32be(self.pos) as u64;
            self.bits = (self.bits << 32) | inbits;
            self.pos += 4;
            self.nbits += 32;
        }
        (self.bits >> (self.nbits - num)) as u32
    }

    #[inline(always)]
    fn consume_bits(&mut self, num: u32) {
        self.nbits -= num;
        self.bits &= (1 << self.nbits) - 1;
    }
}

impl<'a> BitPump for BitPumpPanasonic<'a> {
    fn peek_bits(&mut self, num: u32) -> u32 {
        if num > self.nbits {
            self.nbits += 0x4000 * 8;
            self.pos += 0x4000;
        }
        let mut byte = (self.nbits - num) >> 3 ^ 0x3ff0;
        if self.split {
            byte = (byte + 0x4000 - 0x2008) % 0x4000;
        }
        let bits = self.buffer.u16le(byte as usize + self.pos - 0x4000) as u32;
        (bits >> ((self.nbits - num) & 7)) & (0x0ffffffffu32 >> (32 - num))
    }

    fn consume_bits(&mut self, num: u32) {
        self.nbits -= num;
    }
}
