/// generate tone curve in 12bits for Sony, the length of input curve ndoes should be 4
pub(crate) fn gen_tone_curve_sony(points: &[u16]) -> [u16;4096] {
    let points_12bits = [
        0usize,
        (points[0] >> 2) as usize,
        (points[1] >> 2) as usize,
        (points[2] >> 2) as usize,
        (points[3] >> 2) as usize,
        4095,
    ];

    let mut curve = [0u16; 4096];
    let mut last = 0;
    points_12bits
        .windows(2)
        .enumerate()
        .for_each(|(seg, pair)| {
            let inc = 1 << seg;
            for item in curve.iter_mut().take(pair[1] + 1).skip(pair[0] + 1) {
                let v = last + inc;
                *item = v;
                last = v;
            }
        });

    curve
}

#[cfg(test)]
mod tests {
    use super::gen_tone_curve_sony;

    fn gen_curve_sample(points: &[u16]) -> Box<[u16]> {
        let mut curve: [usize; 6] = [0, 0, 0, 0, 0, 4095];

        for i in 0..4 {
            curve[i + 1] = (points[i] as u32 >> 2) as usize;
        }

        let mut table = vec![0u16; curve[5] + 1];
        for i in 0..5 {
            for j in (curve[i] + 1)..(curve[i + 1] + 1) {
                table[j] = table[j - 1] + (1 << i);
            }
        }

        table.into_boxed_slice()
    }

    #[test]
    fn test() {
        let points = [8000, 11000, 12000, 14000];
        let r1 = gen_curve_sample(&points);
        let r2 = gen_tone_curve_sony(&points);

        assert_eq!(&r1[..], &r2[..]);
    }
}
