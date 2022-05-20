#[derive(Debug, Clone)]
pub struct LookupTable {
    table: Vec<(u16, u16, u16)>,
}

impl LookupTable {
    pub fn new(table: &[u16]) -> LookupTable {
        let mut tbl = vec![(0, 0, 0); table.len()];
        for i in 0..table.len() {
            let center = table[i];
            let lower = if i > 0 { table[i - 1] } else { center };
            let upper = if i < (table.len() - 1) { table[i + 1] } else { center };
            let base = if center == 0 {
                0
            } else {
                center - ((upper - lower + 2) / 4)
            };
            let delta = upper - lower;
            tbl[i] = (center, base, delta);
        }
        LookupTable { table: tbl }
    }

    //  pub fn lookup(&self, value: u16) -> u16 {
    //    let (val, _, _) = self.table[value as usize];
    //    val
    //  }

    #[inline(always)]
    pub fn dither(&self, value: u16, rand: &mut u32) -> u16 {
        let (_, sbase, sdelta) = self.table[value as usize];
        let base = sbase as u32;
        let delta = sdelta as u32;
        let pixel = base + ((delta * (*rand & 2047) + 1024) >> 12);
        *rand = 15700 * (*rand & 65535) + (*rand >> 16);
        pixel as u16
    }
}