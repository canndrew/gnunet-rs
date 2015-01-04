use std::io::IoResult;
use std::io;

pub trait CStringReader: Reader {
  fn read_cstring(&mut self) -> IoResult<String> {
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
      Err(_)  => Err(io::standard_error(io::OtherIoError)),
    }
  }

  fn read_cstring_with_len(&mut self, len: uint) -> IoResult<String> {
    let mut v: Vec<u8> = Vec::with_capacity(len);
    for _ in range(0, len) {
      let b = try!(self.read_u8());
      if b == 0u8 {
        // must not contain embedded NULs
        return Err(io::standard_error(io::OtherIoError));
      }
      v.push(b);
    }
    let b = try!(self.read_u8());
    if b != 0u8 {
      // must be NUL-terminated
      return Err(io::standard_error(io::OtherIoError));
    }
    match String::from_utf8(v) {
      Ok(s)   => Ok(s),
      Err(_)  => Err(io::standard_error(io::OtherIoError)),
    }
  }
}

impl<T> CStringReader for T where T: Reader {}

