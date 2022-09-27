use thiserror::Error;

use maker::RawDecoder;

pub mod data;

pub mod color;
pub mod raw_job;
mod utility;

pub mod maker;
pub mod raw;

#[cfg(feature = "image")]
pub mod export;

pub const BENCH_FLAG: &str = "QUICKRAW_BENCH";
pub const BIT_SHIFT: u32 = 13u32;

#[derive(Debug)]
pub struct ColorConversion {
    white_balance: [i32; 3],
    pub gamma_lut: [u16; 65536],
    color_space: [i32; 9],
}
pub struct RawJob {
    pub file_buffer: Vec<u8>,
    pub decoder: Box<dyn RawDecoder>,
    white_balance: [i32; 3],
    cam_matrix: [f32; 9],
}

#[derive(Clone)]
pub enum DemosaicingMethod {
    None,
    SuperPixel,
    Linear,
}
#[derive(Clone)]
pub enum OutputType {
    Raw8,
    Raw16,
    Image8(String),
    Image16(String),
}
pub enum Input<'a> {
    ByFile(&'a str),
    ByBuffer(Vec<u8>),
}
#[derive(Clone)]
pub struct Output {
    pub demosaicing_method: DemosaicingMethod,
    pub color_space: [f32; 9],
    pub gamma: [f32; 2],
    pub output_type: OutputType,
    pub auto_crop: bool,
    pub auto_rotate: bool
}

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

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("Cannot export the image.")]
    RawFileReadingError(#[from] RawFileReadingError),
    #[error("Cannot create the export object for the file: '{0}'")]
    InvalidFileForNewExport(String),
    #[error("Cannot export image to the file: '{0}'")]
    ErrorWhenExportingFile(String),
    #[error("The {0} image data(len:{1}, width:{2}, height:{3}) is invalid for ImageBuffer.")]
    ImageBufferError(String, usize, usize, usize),
    #[error("Cannot understand the thumbnail image data(len: {0}) for the file: '{1}'")]
    CannotReadThumbnail(usize, String)
}
