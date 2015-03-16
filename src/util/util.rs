use std::path::AsPath;
use std::ffi::{AsOsStr, CString};

use util::error::*;

pub trait CStringReader: Reader {
  fn read_cstring(&mut self) -> Result<String, ReadCStringError> {
    let mut v: Vec<u8> = Vec::new();
    loop {
      let b = try!(self.read_u8());
      if b == 0u8 {
        break;
      }
      v.push(b);
    }
    match String::from_utf8(v) {
      Ok(s)   => Ok(s),
      Err(e)  => Err(ReadCStringError::FromUtf8(e)),
    }
  }

  fn read_cstring_with_len(&mut self, len: usize) -> Result<String, ReadCStringWithLenError> {
    let mut v: Vec<u8> = Vec::with_capacity(len);
    for i in range(0, len) {
      let b = try!(self.read_u8());
      if b == 0u8 {
        // must not contain embedded NULs
        return Err(ReadCStringWithLenError::InteriorNul(i));
      }
      v.push(b);
    }
    let b = try!(self.read_u8());
    if b != 0u8 {
      // must be NUL-terminated
      return Err(ReadCStringWithLenError::NoTerminator);
    }
    match String::from_utf8(v) {
      Ok(s)   => Ok(s),
      Err(e)  => Err(ReadCStringWithLenError::FromUtf8(e)),
    }
  }
}

impl<T> CStringReader for T where T: Reader {}

pub fn to_c_path<P: AsPath + ?Sized>(path: &P) -> Result<CString, ToCPathError> {
  let path = path.as_path();
  let path = match path.as_os_str().to_os_string().into_string() {
    Ok(path)  => path,
    Err(_)    => return Err(ToCPathError::InvalidUnicode),
  };
  let path = match CString::new(path) {
    Ok(path)  => path,
    Err(e)    => return Err(ToCPathError::InteriorNul(e)),
  };
  Ok(path)
}

