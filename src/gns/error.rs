use std::io::IoError;

use service;
use identity;
use FromError;

/// Possible errors returned by the GNS lookup functions.
#[deriving(Show)]
pub enum LookupError {
  /// The specified domain name was too long.
  NameTooLong,
  /// An I/O error occured while talking to the GNS service.
  Io(IoError),
}
error_chain!(IoError, LookupError, Io)

#[deriving(Show)]
pub enum ConnectLookupError {
  /// Failed to connect to the GNS service.
  Connect(service::ConnectError),

  /// The lookup failed.
  Lookup(LookupError),
}
error_chain!(service::ConnectError, ConnectLookupError, Connect)
error_chain!(LookupError, ConnectLookupError, Lookup)

#[deriving(Show)]
pub enum ConnectLookupInMasterError {
  /// Failed to connect to the GNS service and perform the lookup.
  ConnectLookup(ConnectLookupError),
  /// Failed to retrieve the default identity for gns-master from the identity service.
  ConnectGetDefaultEgo(identity::ConnectGetDefaultEgoError),
}
error_chain!(ConnectLookupError, ConnectLookupInMasterError, ConnectLookup)
error_chain!(identity::ConnectGetDefaultEgoError, ConnectLookupInMasterError, ConnectGetDefaultEgo)

