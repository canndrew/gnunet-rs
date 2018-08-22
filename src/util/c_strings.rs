//use std::path::Path;
//use std::ffi::{CString, NulError};
use byteorder::ReadBytesExt;
use std::io::{self, Read};
use std::string::FromUtf8Error;

quick_error! {
    /// Error generated when reading a C-style NUL-terminated string from a service
    #[derive(Debug)]
    pub enum ReadCStringError {
        Io { cause: io::Error } {
            display("There was an I/O error communicating with the service. Specifically: {}", cause)
            cause(cause)
        }

        FromUtf8 { cause: FromUtf8Error } {
            cause(cause)
            from(cause: FromUtf8Error) -> { cause: cause }
            display("The string contained invalid utf-8. Utf8-error: {}", cause)
        }

        Disconnected {
            display("The remote service disconnected unexpectedly")
        }
    }
}
byteorder_error_chain! {ReadCStringError}

quick_error! {
    /// Error generated when attempting to read a C-style NUL-terminated string of known length from a
    /// service.
    #[derive(Debug)]
    pub enum ReadCStringWithLenError {
        Io { cause: io::Error } {
            display("There was an I/O error communicating with the service. Specifically: {}", cause)
            cause(cause)
        }

        FromUtf8 { cause: FromUtf8Error }{
            display("The string contained invalid utf-8. Utf8-error: {}", cause)
            cause(cause)
            from(cause: FromUtf8Error) -> { cause: cause }
        }

        Disconnected {
            display("The remote service disconnected unexpectedly")
        }

        InteriorNul { pos: usize } {
            display("The string contained an interior NUL byte. The offending byte is at position {}", pos)
        }

        NoTerminator {
            display("The string was not NUL-terminated")
        }
    }
}
byteorder_error_chain! {ReadCStringWithLenError}

pub trait ReadCString: Read {
    fn read_c_string(&mut self) -> Result<String, ReadCStringError> {
        let mut v: Vec<u8> = Vec::new();
        loop {
            let b = try!(self.read_u8());
            if b == 0u8 {
                break;
            }
            v.push(b);
        }
        match String::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(e) => Err(ReadCStringError::FromUtf8 { cause: e }),
        }
    }

    fn read_c_string_with_len(&mut self, len: usize) -> Result<String, ReadCStringWithLenError> {
        let mut v: Vec<u8> = Vec::with_capacity(len);
        for i in 0..len {
            let b = try!(self.read_u8());
            if b == 0u8 {
                // must not contain embedded NULs
                return Err(ReadCStringWithLenError::InteriorNul { pos: i });
            }
            v.push(b);
        }
        let b = try!(self.read_u8());
        if b != 0u8 {
            // must be NUL-terminated
            return Err(ReadCStringWithLenError::NoTerminator);
        }
        match String::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(e) => Err(ReadCStringWithLenError::FromUtf8 { cause: e }),
        }
    }
}

impl<T> ReadCString for T where T: Read {}

/*
 *
 *  Currently not used anymore.
 *
quock_error! {
/// A `std::path::Path` could not be converted to a utf-8 CString.
#[derive(Debug)]
pub enum ToCPathError {
  InvalidUnicode {
      display("The path contains invalid unicode")
  }

  InteriorNul { cause: NulError } {
      display("The path contains an interior NUL byte. Specifically: {}", cause)
      cause(cause)
      from(cause: NulError) -> { cause: cause }
  }
}
}

pub fn to_c_path<P: ?Sized>(path: &P) -> Result<CString, ToCPathError>
    where P: AsRef<Path>
{
  let path = path.as_ref();
  let path = match path.as_os_str().to_os_string().into_string() {
    Ok(path)  => path,
    Err(_)    => return Err(ToCPathError::InvalidUnicode),
  };
  let path = match CString::new(path) {
    Ok(path)  => path,
    Err(e)    => return Err(ToCPathError::InteriorNul { cause: e }),
  };
  Ok(path)
}
*/
