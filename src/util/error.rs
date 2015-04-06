use std::old_io::IoError;
use std::string::FromUtf8Error;
use std::ffi::NulError;
use std::fmt;

#[derive(Debug)]
pub enum ReadCStringError {
  Io(IoError),
  FromUtf8(FromUtf8Error),
}
error_chain! {IoError, ReadCStringError, Io}
error_chain! {FromUtf8Error, ReadCStringError, FromUtf8}

#[derive(Debug)]
pub enum ReadCStringWithLenError {
  Io(IoError),
  FromUtf8(FromUtf8Error),
  InteriorNul(usize),
  NoTerminator,
}
error_chain! {IoError, ReadCStringWithLenError, Io}
error_chain! {FromUtf8Error, ReadCStringWithLenError, FromUtf8}

/// A `std::path::Path` could not be converted to a utf-8 CString.
#[derive(Debug)]
pub enum ToCPathError {
  /// The path contains invalid unicode.
  InvalidUnicode,
  /// The path contains an interior NUL byte.
  InteriorNul(NulError),
}
error_chain! {NulError, ToCPathError, InteriorNul}

impl fmt::Display for ReadCStringError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ReadCStringError::Io(ref e)
        => write!(f, "I/O error when reading C string: {}", e),
      &ReadCStringError::FromUtf8(ref e)
        => write!(f, "C string contains invalid utf-8: {}", e),
    }
  }
}

impl fmt::Display for ReadCStringWithLenError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ReadCStringWithLenError::Io(ref e)
        => write!(f, "I/O error when reading sized C string: {}", e),
      &ReadCStringWithLenError::FromUtf8(ref e)
        => write!(f, "Sized C string contains invalid utf-8: {}", e),
      &ReadCStringWithLenError::InteriorNul(i)
        => write!(f, "C string contains interior NUL at position {}", i),
      &ReadCStringWithLenError::NoTerminator
        => write!(f, "NUL terminator expected"),
    }
  }
}

impl fmt::Display for ToCPathError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ToCPathError::InvalidUnicode
        => write!(f, "Path contains invalid unicode"),
      &ToCPathError::InteriorNul(ref e)
        => write!(f, "Path contains an interior NUL: {}", e)
    }
  }
}

