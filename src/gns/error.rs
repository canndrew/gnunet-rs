use std::io::IoError;

use service;
use identity;
use FromError;

/// Possible errors returned by the GNS lookup functions.
#[deriving(Show)]
pub enum LookupError {
  /// The specified domain name was too long.
  LookupError__NameTooLong,
  /// An I/O error occured while talking to the GNS service.
  LookupError__Io(IoError),
}
error_chain!(IoError, LookupError, LookupError__Io)

#[deriving(Show)]
pub enum ConnectLookupError {
  /// Failed to connect to the GNS service.
  ConnectLookupError__Connect(service::ConnectError),

  /// The lookup failed.
  ConnectLookupError__Lookup(LookupError),
}
error_chain!(service::ConnectError, ConnectLookupError, ConnectLookupError__Connect)
error_chain!(LookupError, ConnectLookupError, ConnectLookupError__Lookup)

#[deriving(Show)]
pub enum ConnectLookupInMasterError {
  /// Failed to connect to the GNS service and perform the lookup.
  ConnectLookupInMasterError__ConnectLookup(ConnectLookupError),
  /// Failed to retrieve the default identity for gns-master from the identity service.
  ConnectLookupInMasterError__ConnectGetDefaultEgo(identity::ConnectGetDefaultEgoError),
}
error_chain!(ConnectLookupError, ConnectLookupInMasterError, ConnectLookupInMasterError__ConnectLookup)
error_chain!(identity::ConnectGetDefaultEgoError, ConnectLookupInMasterError, ConnectLookupInMasterError__ConnectGetDefaultEgo)

