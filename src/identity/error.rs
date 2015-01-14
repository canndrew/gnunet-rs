use std::io::IoError;
use std::error::FromError;
use std::string;
use std::fmt;

use service;

/// Errors returned by `IdentityService::get_default_ego`. 
#[derive(Show)]
pub enum GetDefaultEgoError {
  /// The name of the service was too long.
  NameTooLong(String),
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

impl fmt::String for GetDefaultEgoError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &GetDefaultEgoError::NameTooLong(ref s)
          => write!(f, "Name of service \"{}\" is too long for default ego lookup", s),
      &GetDefaultEgoError::Io(ref e)
          => write!(f, "I/O error communicating with identity service during default ego lookup: {}", e),
      &GetDefaultEgoError::ReadMessage(ref e)
          => write!(f, "Error receiving message from identity service during default ego lookup: {}", e),
      &GetDefaultEgoError::ServiceResponse(ref s)
          => write!(f, "Service responded with an error message in response to default ego lookup: {}", s),
      &GetDefaultEgoError::Connect(ref e)
          => write!(f, "Failed to connect to identity service for default ego lookup: {}", e),
      &GetDefaultEgoError::InvalidResponse
          => write!(f, "Service response was incoherent. THIS IS A BUG! Please file a bug report at {}", ::HOMEPAGE),
    }
  }
}

impl fmt::String for ConnectError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ConnectError::Connect(ref e)
          => write!(f, "Failed to contact the identity service: {}", e),
      &ConnectError::Io(ref e)
          => write!(f, "I/O error communicating with the identity service during initial exchange: {}", e),
      &ConnectError::ReadMessage(ref e)
          => write!(f, "Error receiving message from identity service during connection: {}", e),
      &ConnectError::InvalidName(ref e)
          => write!(f, "The identity service sent a non-utf8 encoded name during initial exchange when connecting ({}). THIS IS A BUG. Please file a bug report at {}", e, ::HOMEPAGE),
      &ConnectError::UnexpectedMessageType(n)
          => write!(f, "The identity service sent an unexpected message type ({}) during initial exchange when connecting. THIS IS A BUG. Please file a bug report at {}", n, ::HOMEPAGE),
    }
  }
}

impl fmt::String for ConnectGetDefaultEgoError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ConnectGetDefaultEgoError::GetDefaultEgo(ref e)
          => write!(f, "Connected to identity service but default ego lookup failed: {}", e),
      &ConnectGetDefaultEgoError::Connect(ref e)
          => write!(f, "Failed to connect to identity service to perform default ego lookup: {}", e),
    }
  }
}

