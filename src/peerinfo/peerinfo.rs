use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use libc::{c_char, c_void, size_t};
use std::fmt;
use std::io::{self, Read, Write};
use std::mem::{size_of_val, uninitialized};
use std::str::{from_utf8, FromStr};

use ll;
use service::{self, connect, ReadMessageError, ServiceReader};
use transport::{self, TransportServiceInitError};
use Cfg;
use Hello;

/// The identity of a GNUnet peer.
pub struct PeerIdentity {
    data: ll::Struct_GNUNET_PeerIdentity,
}

impl PeerIdentity {
    pub fn deserialize<R>(r: &mut R) -> Result<PeerIdentity, io::Error>
    where
        R: Read,
    {
        let mut ret: PeerIdentity = unsafe { uninitialized() };
        try!(r.read_exact(&mut ret.data.public_key.q_y[..]));
        Ok(ret)
    }

    pub fn serialize<T>(&self, w: &mut T) -> Result<(), io::Error>
    where
        T: Write,
    {
        w.write_all(&self.data.public_key.q_y[..])
    }
}

quick_error! {
  /// Error generated when attempting to parse a PeerIdentity
  #[derive(Debug)]
  pub enum PeerIdentityFromStrError {
      ParsingFailed {
          display("Failed to parse the string as a PeerIdentity")
      }
  }
}

impl FromStr for PeerIdentity {
    type Err = PeerIdentityFromStrError;

    fn from_str(s: &str) -> Result<PeerIdentity, PeerIdentityFromStrError> {
        unsafe {
            let ret: ll::Struct_GNUNET_PeerIdentity = uninitialized();
            let res = ll::GNUNET_STRINGS_string_to_data(
                s.as_ptr() as *const i8,
                s.len() as size_t,
                ret.public_key.q_y.as_ptr() as *mut c_void,
                ret.public_key.q_y.len() as size_t,
            );
            match res {
                ll::GNUNET_OK => Ok(PeerIdentity { data: ret }),
                _ => Err(PeerIdentityFromStrError::ParsingFailed),
            }
        }
    }
}

quick_error! {
  /// Errors returned by `iterate_peers`.
  #[derive(Debug)]
  pub enum IteratePeersError {
      Io { cause: io::Error } {
        from(cause: io::Error) -> { cause: cause }
          display("There as an I/O error communicating with the peerinfo service. Specifically: {}", cause)
            cause(cause)
      }

      Connect { cause: service::ConnectError } {
        from(cause: service::ConnectError) -> { cause: cause }
          display("Failed to connect to the peerinfo service. Reason: {}", cause)
            cause(cause)
      }
  }
}

/// Iterate over all the currently connected peers.
pub fn iterate_peers(cfg: &Cfg) -> Result<Peers, IteratePeersError> {
    let (sr, mut sw) = try!(connect(cfg, "peerinfo"));

    let msg_length = 8u16;
    let mut mw = sw.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_PEERINFO_GET_ALL);
    mw.write_u32::<BigEndian>(0).unwrap();
    try!(mw.send());
    Ok(Peers { service: sr })
}

pub fn self_id(cfg: &Cfg) -> Result<PeerIdentity, TransportServiceInitError> {
    let hello = try!(transport::self_hello(cfg));
    Ok(hello.id)
}

/// An iterator over all the currently connected peers.
pub struct Peers {
    service: ServiceReader,
}

quick_error! {
    /// Errors returned by `Peers::next`.
    #[derive(Debug)]
    pub enum NextPeerError {
        InvalidResponse {
            display("The response from the gnunet-peerinfo service was incoherent")
        }

        UnexpectedMessageType { ty: u16 } {
            display("The peerinfo service sent an unexpected response message type. Message type {} was not expected", ty)
        }

        Io { cause: io::Error } {
            cause(cause)
            display("There was an I/O error communicating with the peerinfo service. Specifically: {}", cause)
        }

        ReadMessage { cause: ReadMessageError } {
            display("Failed to receive the response from the peerinfo service. Reason: {}", cause)
            cause(cause)
            from(cause: ReadMessageError) -> { cause: cause }
        }

        Disconnected {
            display("The service disconnected unexpectedly")
        }
    }
}
byteorder_error_chain! {NextPeerError}

impl Iterator for Peers {
    type Item = Result<(PeerIdentity, Option<Hello>), NextPeerError>;

    fn next(&mut self) -> Option<Result<(PeerIdentity, Option<Hello>), NextPeerError>> {
        let (tpe, mut mr) = match self.service.read_message() {
            Err(e) => return Some(Err(NextPeerError::ReadMessage { cause: e })),
            Ok(x) => x,
        };
        match tpe {
            ll::GNUNET_MESSAGE_TYPE_PEERINFO_INFO => match mr.read_u32::<BigEndian>() {
                Err(e) => match e.kind() {
                    ::std::io::ErrorKind::UnexpectedEof => Some(Err(NextPeerError::Disconnected)),
                    _ => Some(Err(NextPeerError::Io { cause: e })),
                },
                Ok(x) => match x == 0 {
                    false => Some(Err(NextPeerError::InvalidResponse)),
                    true => match PeerIdentity::deserialize(&mut mr) {
                        Err(e) => Some(Err(NextPeerError::Io { cause: e })),
                        Ok(pi) => {
                            Some(Ok((pi, None)))
                            /*
               * when we have hello parsing
              match mr.eof() {
                true  => Some(Ok(pi, None)),
                false => {

                },
              }
              */
                        }
                    },
                },
            },
            ll::GNUNET_MESSAGE_TYPE_PEERINFO_INFO_END => None,
            x => Some(Err(NextPeerError::UnexpectedMessageType { ty: x })),
        }
    }
}

impl fmt::Debug for PeerIdentity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            const LEN: usize = 52usize;
            assert!(LEN == (size_of_val(&self.data.public_key.q_y) * 8 + 4) / 5);
            let mut enc: [u8; LEN] = uninitialized();
            let res = ll::GNUNET_STRINGS_data_to_string(
                self.data.public_key.q_y.as_ptr() as *const c_void,
                self.data.public_key.q_y.len() as size_t,
                enc.as_mut_ptr() as *mut c_char,
                enc.len() as size_t,
            );
            assert!(!res.is_null());
            fmt::Display::fmt(from_utf8(&enc).unwrap(), f)
        }
    }
}

impl fmt::Display for PeerIdentity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
