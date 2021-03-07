use std::ffi::{CStr, CString};

use crate::errors::{HapiError, Kind, Result};
use crate::session::Session;

// StringBuffer iterators SAFETY: constructing &str and &CStr with unsafe is ok,
// because Houdini string attributes are expected to be valid utf

#[derive(Debug)]
pub struct StringBuffer {
    bytes: Vec<u8>,
}

pub struct StringIter<'a> {
    inner: &'a [u8],
}

pub struct OwnedStringIter {
    inner: Box<[u8]>,
    cursor: usize,
}

impl std::iter::Iterator for OwnedStringIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self
            .inner
            .iter()
            .skip(self.cursor)
            .position(|b| *b == b'\0')
        {
            None => return None,
            Some(pos) => {
                let idx = self.cursor + pos;
                let ret = &self.inner[self.cursor..idx];
                self.cursor = idx + 1;
                Some(String::from_utf8_lossy(ret).to_string())
            }
        }
    }
}

pub struct CStringIter<'a> {
    inner: &'a [u8],
}

impl<'a> StringBuffer {
    pub fn iter_str(&'a self) -> StringIter<'a> {
        StringIter { inner: &self.bytes }
    }

    pub fn iter_cstr(&'a self) -> CStringIter<'a> {
        CStringIter { inner: &self.bytes }
    }
}

impl<'a> std::iter::Iterator for StringIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.iter().position(|c| *c == b'\0') {
            None => None,
            Some(idx) => {
                let ret = &self.inner[..idx];
                self.inner = &self.inner[idx + 1..];
                unsafe { Some(std::str::from_utf8_unchecked(ret)) }
            }
        }
    }
}

impl<'a> std::iter::Iterator for CStringIter<'a> {
    type Item = &'a std::ffi::CStr;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.iter().position(|c| *c == b'\0') {
            None => None,
            Some(idx) => {
                let ret = &self.inner[..idx + 1];
                self.inner = &self.inner[idx + 1..];
                unsafe { Some(CStr::from_bytes_with_nul_unchecked(ret)) }
            }
        }
    }
}

impl std::iter::IntoIterator for StringBuffer {
    type Item = String;
    type IntoIter = OwnedStringIter;

    fn into_iter(self) -> Self::IntoIter {
        OwnedStringIter {
            inner: self.bytes.into_boxed_slice(),
            cursor: 0,
        }
    }
}

pub fn get_string(handle: i32, session: &Session) -> Result<String> {
    unsafe {
        let mut bytes = get_string_bytes(handle, session)?;
        Ok(String::from_utf8_unchecked(bytes))
    }
}

pub fn get_cstring(handle: i32, session: &Session) -> Result<CString> {
    unsafe {
        let mut bytes = get_string_bytes(handle, session)?;
        Ok(CString::from_vec_unchecked(bytes))
    }
}

pub fn get_string_bytes(handle: i32, session: &Session) -> Result<Vec<u8>> {
    unsafe {
        let length = crate::ffi::get_string_buff_len(session, handle)?;
        let buffer = crate::ffi::get_string(session, handle, length)?;
        Ok(buffer)
    }
}

/// Returns a contiguous array of nul-terminated strings
fn get_string_array_bytes(handles: &[i32], session: &Session) -> Result<Vec<u8>> {
    unsafe {
        let length = crate::ffi::get_string_batch_size(handles, session)?;
        if length > 0 {
            Ok(crate::ffi::get_string_batch(length, session)?)
        } else {
            Ok(vec![])
        }
    }
}

#[inline]
pub fn get_string_buffer(handles: &[i32], session: &Session) -> Result<StringBuffer> {
    Ok(StringBuffer {
        bytes: get_string_array_bytes(handles, session)?,
    })
}
