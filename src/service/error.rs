use std::io::IoError;

use FromError;

/// Error that can be generated when attempting to connect to a service.
#[deriving(Show)]
pub enum ConnectError {
  /// Could not load the given config file.
  FailedToLoadConfig,
  /// The config file does not contain information on how to connect to the service.
  NotConfigured,
  /// There was an I/O error communicating with the service.
  Io(IoError),
}
error_chain!(IoError, ConnectError, Io)

#[deriving(Show)]
pub enum ReadMessageError {
  /// There was an I/O error communicating with the service.
  Io(IoError),
  /// The message recieved from the service was too short. *(It is a bug to see this variant)*
  ShortMessage(u16),
}
error_chain!(IoError, ReadMessageError, Io)

