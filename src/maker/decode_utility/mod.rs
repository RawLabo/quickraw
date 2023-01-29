use thiserror::Error;

pub(in super::super) mod huffman;
pub(in super::super) mod bit_pump;
pub(in super::super) mod byte_stream;
pub(in super::super) mod lookup_table;
pub(in super::super) mod ljpeg;

#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("No marker found inside rest of buffer.")]
    ByteStreamNoMarkerFound,
    #[error("LJpeg constructor error: {0}")]
    LJpegErrorConstructor(String),
    #[error("LJpegDecompressing error: {0}")]
    LJpegError(String),
}