use super::*;
use anyhow::Result;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::*;

fn str_from_cchar<'a>(ptr: *mut c_char) -> &'a str {
    let s = unsafe { CStr::from_ptr(ptr) };
    s.to_str().unwrap()
}
fn free_cstring(ptr: *mut c_char) {
    unsafe { CString::from_raw(ptr) };
}
fn gen_cstring(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}
fn gen_empty_cstring() -> *mut c_char {
    CString::new("").unwrap().into_raw()
}

pub trait Free {
    fn free(&mut self);
}

#[repr(C)]
pub struct RustVec {
    ptr: *mut c_uchar,
    len: c_uint,
    capacity: c_uint,
}

impl RustVec {
    fn new_empty() -> Self {
        let x: Vec<c_uchar> = vec![];
        RustVec {
            ptr: x.as_ptr() as *mut c_uchar,
            len: x.len() as c_uint,
            capacity: x.capacity() as c_uint,
        }
    }

    fn new(x: Vec<u8>) -> Self {
        let result = RustVec {
            ptr: x.as_ptr() as *mut c_uchar,
            len: x.len() as c_uint,
            capacity: x.capacity() as c_uint,
        };
        std::mem::forget(x);
        result
    }

    fn free(&mut self) {
        unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
    }
}

#[repr(C)]
pub struct QuickrawResponse<T> {
    has_error: bool,
    error_msg: *mut c_char,
    content: T,
}

impl<T: Default> QuickrawResponse<T> {
    fn new(x: Result<T>) -> Self {
        let (has_error, error, content) = match x {
            Ok(x) => (false, gen_empty_cstring(), x),
            Err(e) => (true, gen_cstring(e.to_string()), T::default()),
        };
        QuickrawResponse {
            has_error,
            error_msg: error,
            content,
        }
    }
}
impl<T: Free> Free for QuickrawResponse<T> {
    fn free(&mut self) {
        free_cstring(self.error_msg);
        self.content.free();
    }
}

#[repr(C)]
pub struct BasicInfo {
    exif: *mut c_char,
    thumbnail: RustVec,
    orientation: c_uchar,
}
impl Default for BasicInfo {
    fn default() -> Self {
        BasicInfo {
            exif: gen_empty_cstring(),
            thumbnail: RustVec::new_empty(),
            orientation: 0,
        }
    }
}
impl BasicInfo {
    fn new(exif: String, thumbnail: RustVec, orientation: c_uchar) -> Self {
        BasicInfo {
            exif: gen_cstring(exif),
            thumbnail,
            orientation,
        }
    }
}
impl Free for BasicInfo {
    fn free(&mut self) {
        free_cstring(self.exif);
        self.thumbnail.free();
    }
}

#[no_mangle]
pub extern "C" fn quickraw_load_basicinfo(
    cpath: *mut c_char,
    with_thumbnail: bool,
) -> QuickrawResponse<BasicInfo> {
    QuickrawResponse::new(load_basicinfo(cpath, with_thumbnail))
}
fn load_basicinfo(cpath: *mut c_char, with_thumbnail: bool) -> Result<BasicInfo> {
    let path = str_from_cchar(cpath);
    let exif = Export::export_exif_info(Input::ByFile(path))?;
    let s = exif.stringify_all()?;
    let (thumbnail, orientation) = if with_thumbnail {
        let buffer = fs::read(path)?;
        let (data, orientation) = Export::export_thumbnail_data(&buffer)?;
        (RustVec::new(data.to_vec()), orientation as u8)
    } else {
        (RustVec::new_empty(), 0)
    };
    Ok(BasicInfo::new(s, thumbnail, orientation))
}

#[no_mangle]
pub extern "C" fn quickraw_free_basicinfo(mut response: QuickrawResponse<BasicInfo>) {
    response.free();
}

#[repr(C)]
pub struct Image {
    data: RustVec,
    width: c_uint,
    height: c_uint,
}
impl Free for Image {
    fn free(&mut self) {
        self.data.free();
    }
}
impl Default for Image {
    fn default() -> Self {
        Image {
            data: RustVec::new_empty(),
            width: 0,
            height: 0,
        }
    }
}
