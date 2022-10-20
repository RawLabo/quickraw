#![allow(dead_code)]

use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Image {
    pub width: usize,
    pub height: usize,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl Image {
    #[wasm_bindgen(getter)]
    pub fn data(self) -> Vec<u8> {
        self.data
    }
}

#[wasm_bindgen]
pub fn load_image(input: Vec<u8>) -> Image {
    let export = Export::new(
        Input::ByBuffer(input),
        Output::new(
            DemosaicingMethod::Linear,
            data::XYZ2SRGB,
            data::GAMMA_LINEAR,
            OutputType::Raw8,
            false,
            false,
        ),
    )
    .unwrap();

    let (data, width, height) = export.export_8bit_image();
    Image {
        data,
        width,
        height,
    }
}
