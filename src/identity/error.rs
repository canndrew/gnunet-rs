use std::io::IoError;
use std::error::FromError;
use std::string;

use service;

/// Errors returned by `IdentityService::get_default_ego`. 
#[derive(Show)]
pub enum GetDefaultEgoError {
  /// The name of the service was too long.
  NameTooLong,
  /// An I/O error occured while communicating with the identity service.
  Io(IoError),
  /// Failed to read a message from the server.
  ReadMessage(service::ReadMessageError),
  /// The service responded with an error message.
  ServiceResponse(String),
  /// Failed to connect to the identity service.
  Connect(ConnectError),
  /// The service response was incoherent. You should file a bug-report if you encounter this
  /// variant.
  InvalidResponse,
}
error_chain! {ConnectError, GetDefaultEgoError, Connect}
error_chain! {IoError, GetDefaultEgoError, Io}
error_chain! {service::ReadMessageError, GetDefaultEgoError, ReadMessage}

/// Errors returned by `IdentityService::connect`
#[derive(Show)]
pub enum ConnectError {
  /// Failed to connect to the service.
  Connect(service::ConnectError),
  /// There was an I/O error communicating with the service.
  Io(IoError),
  /// Failed to read a message from the service.
  ReadMessage(service::ReadMessageError),
  /// The service responded with an invalid utf-8 name. *(It is a bug to see this variant)*
  InvalidName(string::FromUtf8Error),
  /// Received an unexpected message from the service. *(It is a bug to see this variant)*
  UnexpectedMessageType(u16),
}
error_chain! {service::ConnectError, ConnectError, Connect}
error_chain! {IoError, ConnectError, Io}
error_chain! {service::ReadMessageError, ConnectError, ReadMessage}

/// Errors returned by `identity::get_default_ego`
#[derive(Show)]
pub enum ConnectGetDefaultEgoError {
  /// Ego lookup failed.
  GetDefaultEgo(GetDefaultEgoError),
  /// Failed to connect to the service and perform initialization.
  Connect(ConnectError),
}
error_chain! {GetDefaultEgoError, ConnectGetDefaultEgoError, GetDefaultEgo}
error_chain! {ConnectError, ConnectGetDefaultEgoError, Connect}

