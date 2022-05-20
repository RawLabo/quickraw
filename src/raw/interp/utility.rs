pub trait ImageOp {
    fn get_pixel(&self, i: usize) -> i32;
    fn avg<const N: usize>(&self, indexes: [usize; N]) -> i32;
}

impl ImageOp for &[u16] {
    #[inline(always)]
    fn get_pixel(&self, i: usize) -> i32 {
        unsafe { *self.get_unchecked(i) as i32 }
    }
    #[inline(always)]
    fn avg<const N: usize>(&self, indexes: [usize; N]) -> i32 {
        indexes.iter().map(|&i| self.get_pixel(i)).sum::<i32>() / N as i32
    }
}