use std::io::IoError;

use service;
use FromError;

/// Errors returned by `IdentityService::get_default_ego`. 
#[deriving(Show)]
pub enum GetDefaultEgoError {
  /// The name of the service was too long.
  GetDefaultEgoError__NameTooLong,
  /// An I/O error occured while communicating with the identity service.
  GetDefaultEgoError__Io(IoError),
  /// Failed to read a message from the server.
  GetDefaultEgoError__ReadMessage(service::ReadMessageError),
  /// The service responded with an error message.
  GetDefaultEgoError__ServiceResponse(String),
  /// Failed to connect to the identity service.
  GetDefaultEgoError__Connect(ConnectError),
  /// The service response was incoherent. You should file a bug-report if you encounter this
  /// variant.
  GetDefaultEgoError__InvalidResponse,
}
error_chain!(ConnectError, GetDefaultEgoError, GetDefaultEgoError__Connect)
error_chain!(IoError, GetDefaultEgoError, GetDefaultEgoError__Io)
error_chain!(service::ReadMessageError, GetDefaultEgoError, GetDefaultEgoError__ReadMessage)

/// Errors returned by `IdentityService::connect`
#[deriving(Show)]
pub enum ConnectError {
  /// Failed to connect to the service.
  ConnectError__Connect(service::ConnectError),
  /// There was an I/O error communicating with the service.
  ConnectError__Io(IoError),
  /// Failed to read a message from the service.
  ConnectError__ReadMessage(service::ReadMessageError),
  /// The service responded with an invalid utf-8 name. *(It is a bug to see this variant)*
  ConnectError__InvalidName(Vec<u8>),
  /// Received an unexpected message from the service. *(It is a bug to see this variant)*
  ConnectError__UnexpectedMessageType(u16),
}
error_chain!(service::ConnectError, ConnectError, ConnectError__Connect)
error_chain!(IoError, ConnectError, ConnectError__Io)
error_chain!(service::ReadMessageError, ConnectError, ConnectError__ReadMessage)

/// Errors returned by `identity::get_default_ego`
#[deriving(Show)]
pub enum ConnectGetDefaultEgoError {
  /// Ego lookup failed.
  ConnectGetDefaultEgoError__GetDefaultEgo(GetDefaultEgoError),
  /// Failed to connect to the service and perform initialization.
  ConnectGetDefaultEgoError__Connect(ConnectError),
}
error_chain!(GetDefaultEgoError, ConnectGetDefaultEgoError, ConnectGetDefaultEgoError__GetDefaultEgo)
error_chain!(ConnectError, ConnectGetDefaultEgoError, ConnectGetDefaultEgoError__Connect)

