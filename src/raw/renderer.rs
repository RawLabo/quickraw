use crate::ColorConversion;

use super::{interp::*, *};

impl DecodedImage {
    pub(crate) fn super_pixel_render<T>(&self, cc: &ColorConversion, cast_fn: fn(u16) -> T) -> (Vec<T>, usize, usize) {
        let image = self.image.as_slice();
        let w = self.width;
        let h = self.height;

        let iter = image.chunks_exact(w * 2).enumerate();

        macro_rules! render {
            ($cfa:tt) => {
                iter.flat_map(|(block_index, block)| {
                    let offset = block_index * w * 2;
                    block.iter().take(w).enumerate().step_by(2).map(move |(i, &v)| {
                        let i = offset + i;
                        let rgb = SuperPixel::$cfa(i, v, image, w);
                        cc.convert(rgb)
                    })
                })
                .flatten()
                .map(cast_fn)
                .collect()
            };
        }

        let data: Vec<T> = match self.cfa_pattern {
            CFAPattern::RGGB => render!(rggb),
            CFAPattern::BGGR => render!(bggr),
            CFAPattern::GRBG => render!(grbg),
            CFAPattern::GBRG => render!(gbrg),
            CFAPattern::XTrans0 | CFAPattern::XTrans1 => unimplemented!(),
        };

        (data, w / 2, h / 2)
    }

    pub(crate) fn linear_render<T>(&self, cc: &ColorConversion, cast_fn: fn(u16) -> T) -> (Vec<T>, usize, usize) {
        let image = self.image.as_slice();
        let w = self.width;
        let h = self.height;

        let iter = image.iter().enumerate().map(|(i, &v)| PixelInfo::new(i, v, w, h));

        macro_rules! render {
            ($cfa:tt) => {
                iter.flat_map(|pi| {
                    let rgb = Linear::$cfa(pi, image, w);
                    cc.convert(rgb)
                })
                .map(cast_fn)
                .collect()
            };
        }

        let data: Vec<T> = match self.cfa_pattern {
            CFAPattern::RGGB => render!(rggb),
            CFAPattern::BGGR => render!(bggr),
            CFAPattern::GRBG => render!(grbg),
            CFAPattern::GBRG => render!(gbrg),
            CFAPattern::XTrans0 => render!(xtrans0),
            CFAPattern::XTrans1 => render!(xtrans1),
        };

        (data, w, h)
    }

    pub(crate) fn no_demosaic_render<T>(&self, cc: &ColorConversion, cast_fn: fn(u16) -> T) -> (Vec<T>, usize, usize) {
        let image = self.image.as_slice();
        let w = self.width;
        let h = self.height;

        let iter = image.iter().enumerate().map(|(i, &v)| PixelInfo::new(i, v, w, h));

        macro_rules! render {
            ($cfa:tt) => {
                iter.flat_map(|pi| {
                    let (r, g, b) = None::$cfa(pi, image, w);
                    [
                        cc.gamma_lut[r as usize],
                        cc.gamma_lut[g as usize],
                        cc.gamma_lut[b as usize],
                    ]
                })
                .map(cast_fn)
                .collect()
            };
        }

        let data: Vec<T> = match self.cfa_pattern {
            CFAPattern::RGGB => render!(rggb),
            CFAPattern::BGGR => render!(bggr),
            CFAPattern::GRBG => render!(grbg),
            CFAPattern::GBRG => render!(gbrg),
            CFAPattern::XTrans0 | CFAPattern::XTrans1 => unimplemented!(),
        };

        (data, w, h)
    }
}
