use std::old_io::IoError;
use std::fmt;

use service::{ConnectError, ReadMessageError};

/// Errors returned by `iterate_peers`.
#[derive(Debug)]
pub enum IteratePeersError {
  Io(IoError),
  Connect(ConnectError),
}
error_chain! {IoError, IteratePeersError, Io}
error_chain! {ConnectError, IteratePeersError, Connect}

/// Errors returned by `Peers::next`.
#[derive(Debug)]
pub enum NextPeerError {
  /// The response from the gnunet-peerinfo service was incoherent.
  InvalidResponse,
  /// The gnunet-peerinfo service gave an unexpected response.
  UnexpectedMessageType(u16),
  /// There was an I/O error communicating with the gnunet-peerinfo service.
  Io(IoError),
  /// There was an error receiving the response from the gnunet-peerinfo service.
  ReadMessage(ReadMessageError),
}
error_chain! {IoError, NextPeerError, Io}
error_chain! {ReadMessageError, NextPeerError, ReadMessage}

impl fmt::Display for IteratePeersError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &IteratePeersError::Io(ref e)
        => write!(f, "I/O error connecting to service: {}", e),
      &IteratePeersError::Connect(ref e)
        => write!(f, "Error connecting to service: {}", e),
    }
  }
}

impl fmt::Display for NextPeerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &NextPeerError::InvalidResponse
        => write!(f, "The service sent a malformed response"),
      &NextPeerError::UnexpectedMessageType(e)
        => write!(f, "The service responsed with an unexpected message type id ({})", e),
      &NextPeerError::Io(ref e)
        => write!(f, "I/O error communicating with service: {}", e),
      &NextPeerError::ReadMessage(ref e)
        => write!(f, "Error parsing service response: {}", e),
    }
  }
}

