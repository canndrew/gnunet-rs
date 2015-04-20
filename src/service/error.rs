use std::io;
use std::fmt;
use byteorder;

/// Error that can be generated when attempting to connect to a service.
#[derive(Debug)]
pub enum ConnectError {
  /// The config file does not contain information on how to connect to the service.
  NotConfigured,
  /// There was an I/O error communicating with the service.
  Io(io::Error),
}
error_chain! {io::Error, ConnectError, Io}

#[derive(Debug)]
pub enum ReadMessageError {
  /// There was an I/O error communicating with the service.
  Io(io::Error),
  /// The message recieved from the service was too short. *(It is a bug to see this variant)*
  ShortMessage(u16),
  /// The service disconnected.
  Disconnected,
}
error_chain! {io::Error, ReadMessageError, Io}
byteorder_error_chain! {ReadMessageError}

impl fmt::Display for ConnectError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ConnectError::NotConfigured
          => write!(f, "The configuration does not contain sufficient information to connect to the service"),
      &ConnectError::Io(ref e)
          => write!(f, "I/O error connecting to service: {}", e),
    }
  }
}

impl fmt::Display for ReadMessageError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ReadMessageError::Io(ref e)
          => write!(f, "I/O error receiving data from service: {}", e),
      &ReadMessageError::ShortMessage(ref b)
          => write!(f, "Invalid message size in message header ({} bytes). THIS IS PROBABLY A BUG! Please submit a bug report at {}.", b, ::HOMEPAGE),
      &ReadMessageError::Disconnected
          => write!(f, "The service unexpectedly disconnected."),
    }
  }
}

