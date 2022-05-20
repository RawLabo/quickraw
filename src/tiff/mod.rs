use std::collections::HashMap;
use thiserror::Error;

mod value;
pub mod parsed_raw_info;
pub mod parser;
pub mod utility;

#[derive(Debug, Clone)]
pub enum Value {
    U16(u16),
    U32(u32),
    R64(f64),
    Str(String),
}

pub struct ParsedRawInfo {
    pub is_le: bool,
    pub content: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum OffsetType {
    Bytes(isize),
    Address,
    PrevField(&'static str)
}

#[derive(Debug, Copy, Clone)]
pub enum CondType {
    LT, EQ, GT, EXIST
}

#[derive(Debug, Clone)]
pub enum ExifTask {
    Tiff(Vec<ExifTask>),

    Condition {
        cond: (CondType, &'static str, u32),
        left: Vec<ExifTask>,
        right: Vec<ExifTask>
    },
    Offset(OffsetType, Vec<ExifTask>),
    Jump {
        tag: u16,
        is_optional: bool,
        tasks: Vec<ExifTask>
    },
    JumpNext(Vec<ExifTask>),
    Scan {
        marker: &'static [u8],
        name: Option<&'static str>,
        tasks: Vec<ExifTask>
    },
    SonyDecrypt {
        offset_tag: u16,
        len_tag: u16,
        key_tag: u16,
        tasks: Vec<ExifTask>,
    },
    TagItem {
        tag: u16,
        name: &'static str,
        len: Option<&'static str>,
        is_optional: bool,
        is_value_u16: bool
    },
    OffsetItem {
        offset: usize,
        name: &'static str,
        t: Value,
    },
}

pub struct Parser<'a> {
    is_le: bool, // is little endian
    buffer: &'a [u8],
    offset: usize,
    entires: HashMap<u16, &'a [u8]>,
    next_offset: usize
}

pub struct TaskInfo {
    pub cam2xyz_matrix: [f32; 9],
    pub task: ExifTask,
}

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Parsing exif error.")]
    RawInfoError(#[from] RawInfoError),
    #[error("The byte order of tiff header {0:#02x?} is invalid.")]
    InvalidTiffHeaderByteOrder(u16),
    #[error("The tag {0:#02x?} was not found.")]
    TagNotFound(u16),
    #[error("The start task should be Tiff or JPEG.")]
    InvalidStartTask,
    #[error("Scan failed to find '{0:#02x?}'.")]
    ScanFailed(&'static [u8])
}

#[derive(Error, Debug)]
pub enum RawInfoError {
    #[error("The raw has invalid info value.")]
    ValueError(#[from] ValueError),
    #[error("The collector from decorder does not contain the '{0}' field.")]
    FieldNotFound(String),
    #[error("The value's type of the field:'{0}' is invalid")]
    FieldValueIsInvalid(String),
}

#[derive(Error, Debug)]
pub enum ValueError {
    #[error("The value's type is not '{0}'")]
    ValueTypeIsNotDesired(&'static str)
}
