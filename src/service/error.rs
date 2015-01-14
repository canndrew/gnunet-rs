use std::io::IoError;
use std::error::FromError;
use std::fmt;

/// Error that can be generated when attempting to connect to a service.
#[derive(Show)]
pub enum ConnectError {
  /// The config file does not contain information on how to connect to the service.
  NotConfigured,
  /// There was an I/O error communicating with the service.
  Io(IoError),
}
error_chain! {IoError, ConnectError, Io}

#[derive(Show)]
pub enum ReadMessageError {
  /// There was an I/O error communicating with the service.
  Io(IoError),
  /// The message recieved from the service was too short. *(It is a bug to see this variant)*
  ShortMessage(u16),
}
error_chain! {IoError, ReadMessageError, Io}

impl fmt::String for ConnectError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ConnectError::NotConfigured
          => write!(f, "The configuration does not contain sufficient information to connect to the service"),
      &ConnectError::Io(ref e)
          => write!(f, "I/O error connecting to service: {}", e),
    }
  }
}

impl fmt::String for ReadMessageError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ReadMessageError::Io(ref e)
          => write!(f, "I/O error receiving data from service: {}", e),
      &ReadMessageError::ShortMessage(ref b)
          => write!(f, "Invalid message size in message header ({} bytes). THIS IS PROBABLY A BUG! Please submit a bug report at {}.", b, ::HOMEPAGE),
    }
  }
}

