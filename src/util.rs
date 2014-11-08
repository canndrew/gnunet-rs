use std::io::{BytesReader, IoResult};
use std::io;

pub trait CStringReader: Reader {
  fn read_cstring(&mut self, capacity: Option<uint>) -> IoResult<String> {
    let mut v: Vec<u8> = match capacity {
      Some(c) => Vec::with_capacity(c),
      None    => Vec::new(),
    };
    for r in self.bytes() {
      let b = try!(r);
      if b == 0u8 {
        break;
      }
      v.push(b);
    }
    match String::from_utf8(v) {
      Ok(s)   => Ok(s),
      Err(_)  => Err(io::standard_error(io::EndOfFile)),
    }
  }
}

impl<T> CStringReader for T where T: Reader {}

