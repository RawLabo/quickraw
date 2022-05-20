use crate::tiff::utility::{GetBytesFromInt, GetNumFromBytes};

pub fn sony_decrypt(data: &[u8], mut key: u32, is_le: bool) -> Vec<u8> {
    let mut pad = [0u32; 128];
    for item in pad.iter_mut().take(4) {
        key = key.wrapping_mul(48828125).wrapping_add(1);
        *item = key;
    }
    pad[3] = pad[3] << 1 | (pad[0] ^ pad[2]) >> 31;
    for i in 4..127 {
        pad[i] = (pad[i - 4] ^ pad[i - 2]) << 1 | (pad[i - 3] ^ pad[i - 1]) >> 31;
    }
    for item in pad.iter_mut().take(127) {
        *item = item.swap_bytes();
    }

    data.chunks_exact(4)
        .map(|x| x.u32(is_le, 0))
        .zip(127..)
        .flat_map(|(x, p)| {
            pad[p & 127] = pad[(p + 1) & 127] ^ pad[(p + 65) & 127];
            (x ^ pad[p & 127]).to_bytes(is_le)
        })
        .collect::<Vec<u8>>()
}

