use std::mem;
use std::io::Read;
use byteorder;

fn uninitialised_vec(len: usize) -> Vec<u8> {
  let mut buf: Vec<u8> = Vec::with_capacity(len);
  let ret = unsafe { Vec::from_raw_parts(buf.as_mut_ptr(), len, buf.capacity()) };
  mem::forget(buf);
  ret
}

pub trait ReadUtil: Read {
  fn read_exact_alloc(&mut self, len: usize) -> Result<Vec<u8>, byteorder::Error> {
    let mut ret = uninitialised_vec(len);
    try!(self.read_exact(&mut ret[..]));
    Ok(ret)
  }
}

impl<R> ReadUtil for R where R: Read {
}

