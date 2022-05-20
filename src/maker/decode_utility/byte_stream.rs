use crate::tiff::utility::GetNumFromBytes;

use super::DecodingError;

#[derive(Debug, Copy, Clone)]
pub struct ByteStream<'a> {
    buffer: &'a [u8],
    pos: usize,
    is_le: bool,
}

impl<'a> ByteStream<'a> {
    pub fn new(src: &'a [u8], is_le: bool) -> ByteStream {
        ByteStream {
            buffer: src,
            pos: 0,
            is_le,
        }
    }

    #[inline(always)]
    pub fn get_pos(&self) -> usize {
        self.pos
    }

    #[inline(always)]
    pub fn peek_u8(&self) -> u8 {
        self.buffer[self.pos]
    }
    #[inline(always)]
    pub fn get_u8(&mut self) -> u8 {
        let val = self.peek_u8();
        self.pos += 1;
        val
    }

    #[inline(always)]
    pub fn peek_u16(&self) -> u16 {
        self.buffer.u16(self.is_le, self.pos)
    }
    #[inline(always)]
    pub fn get_u16(&mut self) -> u16 {
        let val = self.peek_u16();
        self.pos += 2;
        val
    }

    #[inline(always)]
    pub fn consume_bytes(&mut self, num: usize) {
        self.pos += num
    }

    #[inline(always)]
    pub fn skip_to_marker(&mut self) -> Result<usize, DecodingError> {
        let mut skip_count = 0;
        while !(self.buffer[self.pos] == 0xFF && self.buffer[self.pos + 1] != 0 && self.buffer[self.pos + 1] != 0xFF) {
            self.pos += 1;
            skip_count += 1;
            if self.pos >= self.buffer.len() {
                return Err(DecodingError::ByteStreamNoMarkerFound);
            }
        }
        self.pos += 1; // Make the next byte the marker
        Ok(skip_count + 1)
    }
}