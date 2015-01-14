use std::io::IoError;
use std::error::FromError;
use std::fmt;

use service;
use identity;
use ll;

/// Possible errors returned by the GNS lookup functions.
#[derive(Show)]
pub enum LookupError {
  /// The specified domain name was too long.
  NameTooLong(String),
  /// An I/O error occured while talking to the GNS service.
  Io(IoError),
}
error_chain! {IoError, LookupError, Io}

#[derive(Show)]
pub enum ConnectLookupError {
  /// Failed to connect to the GNS service.
  Connect(service::ConnectError),

  /// The lookup failed.
  Lookup(LookupError),
}
error_chain! {service::ConnectError, ConnectLookupError, Connect}
error_chain! {LookupError, ConnectLookupError, Lookup}

#[derive(Show)]
pub enum ConnectLookupInMasterError {
  /// Failed to connect to the GNS service and perform the lookup.
  GnsLookup(ConnectLookupError),
  /// Failed to retrieve the default identity for gns-master from the identity service.
  IdentityGetDefaultEgo(identity::ConnectGetDefaultEgoError),
}
error_chain! {ConnectLookupError, ConnectLookupInMasterError, GnsLookup}
error_chain! {identity::ConnectGetDefaultEgoError, ConnectLookupInMasterError, IdentityGetDefaultEgo}

impl fmt::String for LookupError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &LookupError::NameTooLong(ref name)
          => write!(f, "The domain name \"{}\" is too long to lookup. Maximum length is {} bytes", name, ll::GNUNET_DNSPARSER_MAX_NAME_LENGTH),
      &LookupError::Io(ref e)
          => write!(f, "I/O error communicating with GNS service to perform lookup: {}", e),
    }
  }
}

impl fmt::String for ConnectLookupError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ConnectLookupError::Connect(ref e)
          => write!(f, "Connect error during connect and lookup: {}", e),
      &ConnectLookupError::Lookup(ref e)
          => write!(f, "Lookup error during connect and lookup: {}", e),
    }
  }
}

impl fmt::String for ConnectLookupInMasterError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ConnectLookupInMasterError::GnsLookup(ref e)
          => write!(f, "Lookup with GNS service failed: {}", e),
      &ConnectLookupInMasterError::IdentityGetDefaultEgo(ref e)
          => write!(f, "Failed to retrieve gns-master default ego from identity service: {}", e),
    }
  }
}

