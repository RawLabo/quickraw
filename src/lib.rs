//! A pure rust library to handle camera raw files.
//! 
//! **quickraw** is a pure rust library to decode and renderer image from camera raw files.
//! 
//! ## Examples
//! #### Export thumbnail
//! ```no_run
//! use quickraw::Export;
//! 
//! let raw_data = std::fs::read("sample.ARW").unwrap();
//! let (thumbnail_data, orientation) = Export::export_thumbnail_data(&raw_data).unwrap();
//! 
//! // notice that this function is available on feature `image` only.
//! quickraw::Export::export_thumbnail_to_file("sample.ARW", "sample.thumbnail.jpg").unwrap();
//! ```
//! 
//! #### Get EXIF data
//! ```no_run
//! use quickraw::Export;
//! let info = Export::export_exif_info(Input::ByFile("sample.ARW")).unwrap();
//! 
//! // info is a `quickexif::ParsedInfo` type, for more info please check https://docs.rs/quickexif
//! let width = info.usize("width").unwrap();
//! ```
//! #### Export image
//! ```no_run
//! use quickraw::{data, DemosaicingMethod, Input, Output, Export, OutputType};
//! 
//! let demosaicing_method = DemosaicingMethod::Linear;
//! let color_space = data::XYZ2SRGB;
//! let gamma = data::GAMMA_SRGB;
//! let output_type = OutputType::Raw16;
//! let auto_crop = false;
//! let auto_rotate = false;
//! 
//! let export_job = Export::new(
//!     Input::ByFile("sample.ARW"),
//!     Output::new(
//!         demosaicing_method,
//!         color_space,
//!         gamma,
//!         output_type,
//!         auto_crop,
//!         auto_rotate,
//!     ),
//! ).unwrap();
//! 
//! let (image, width, height) = export_job.export_16bit_image();
//! 
//! // or you can also export an image with quality(only works when the output type is JPEG).
//! // notice that this function is available on feature `image` only.
//! export_job.export_image(92).unwrap();
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

/// A flag to enable benchmark for several key processes.
pub const BENCH_FLAG: &str = "QUICKRAW_BENCH";

use thiserror::Error;

pub mod data;

mod color;
mod utility;

mod maker;
mod raw;

mod decode;
pub use decode::new_image_from_file;
pub use decode::new_image_from_buffer;

pub mod export;
pub use export::Export;

const BIT_SHIFT: u32 = 13u32;

#[derive(Debug)]
struct ColorConversion {
    white_balance: [i32; 3],
    gamma_lut: [u16; 65536],
    color_space: [i32; 9],
}

/// All the demosaicing method currently supported.
#[derive(Clone)]
pub enum DemosaicingMethod {
    None,
    SuperPixel,
    Linear,
}

/// Decides if the output should be 8bit or 16bit.
#[derive(Clone)]
pub enum OutputType {
    Raw8,
    Raw16,
    Image8(String),
    Image16(String),
}

/// Chooses the input from a file or a buffer.
pub enum Input<'a> {
    ByFile(&'a str),
    ByBuffer(Vec<u8>),
}

/// Contains options for image rendering.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Output {
    demosaicing_method: DemosaicingMethod,
    color_space: [f32; 9],
    gamma: [f32; 2],
    output_type: OutputType,
    auto_crop: bool,
    auto_rotate: bool,
}
impl Output {
    pub fn new(
        demosaicing_method: DemosaicingMethod,
        color_space: [f32; 9],
        gamma: [f32; 2],
        output_type: OutputType,
        auto_crop: bool,
        auto_rotate: bool,
    ) -> Output {
        Output {
            demosaicing_method,
            color_space,
            gamma,
            output_type,
            auto_crop,
            auto_rotate,
        }
    }
}

/// Errors of raw file reading.
#[derive(Error, Debug)]
pub enum RawFileReadingError {
    #[error("Exif parsing error.")]
    ExifParseError(#[from] quickexif::parser::Error),
    #[error("Exif parsed info error.")]
    ExifParseInfoError(#[from] quickexif::parsed_info::Error),
    #[error("Cannot read the raw file.")]
    DecodingError(#[from] maker::DecodingError),
    #[error("The file '{0}' is not existed.")]
    FileNotExisted(String),
    #[error("The metadata of file '{0}' cannot be read.")]
    FileMetadataReadingError(String),
    #[error("The content of file '{0}' cannot be read.")]
    FileContentReadingError(String),
    #[error("Cannot read Make info from this raw file.")]
    CannotReadMake,
    #[error("Cannot read Model info from this raw file.")]
    CannotReadModel,
    #[error("This raw file from maker: '{0}' is not supported yet.")]
    MakerIsNotSupportedYet(String),
    #[error("This raw file model: '{0}' is not supported yet.")]
    ModelIsNotSupportedYet(String),
}

