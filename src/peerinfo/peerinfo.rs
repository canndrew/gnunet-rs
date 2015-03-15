use std::mem::{uninitialized, size_of_val};
use std::old_io::IoResult;
use std::fmt;
use std::str::from_utf8;
use libc::{c_void, c_char, size_t};

use ll;
use Configuration;
use service::{connect, ServiceReader};
use Hello;

use peerinfo::error::*;

pub struct PeerIdentity {
  data: ll::Struct_GNUNET_PeerIdentity,
}

impl PeerIdentity {
  pub fn deserialize<R>(r: &mut R) -> IoResult<PeerIdentity> where R: Reader {
    let mut ret: PeerIdentity = unsafe { uninitialized() };
    try!(r.read_at_least(ret.data.public_key.q_y.len(), &mut ret.data.public_key.q_y));
    Ok(ret)
  }
}

pub fn iterate_peers(cfg: &Configuration) -> Result<Peers, IteratePeersError> {
  let (sr, mut sw) = try!(connect(cfg, "peerinfo"));
  
  let msg_length = 8u16;
  let mut mw = sw.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_PEERINFO_GET_ALL);
  try!(mw.write_be_u32(0));
  try!(mw.send());
  Ok(Peers {
    service: sr,
  })
} 

pub struct Peers {
  service: ServiceReader,
}

impl Iterator for Peers {
  type Item = Result<(PeerIdentity, Option<Hello>), NextPeerError>;

  fn next(&mut self) -> Option<Result<(PeerIdentity, Option<Hello>), NextPeerError>> {
    let (tpe, mut mr) = match self.service.read_message() {
      Err(e)  => return Some(Err(NextPeerError::ReadMessage(e))),
      Ok(x)   => x,
    };
    match tpe {
      ll::GNUNET_MESSAGE_TYPE_PEERINFO_INFO => match mr.read_be_u32() {
        Err(e)  => Some(Err(NextPeerError::Io(e))),
        Ok(x)   => match x == 0 {
          false => Some(Err(NextPeerError::InvalidResponse)),
          true  => match PeerIdentity::deserialize(&mut mr) {
            Err(e)  => Some(Err(NextPeerError::Io(e))),
            Ok(pi)  => {
              Some(Ok((pi, None)))
              /*
               * when we have hello parsing
              match mr.eof() {
                true  => Some(Ok(pi, None)),
                false => {

                },
              }
              */
            },
          },
        },
      },
      ll::GNUNET_MESSAGE_TYPE_PEERINFO_INFO_END => None,
      x => Some(Err(NextPeerError::UnexpectedMessageType(x))),
    }
  }
}

impl fmt::Debug for PeerIdentity {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unsafe {
      const LEN: usize = 52usize;
      assert!(LEN == (size_of_val(&self.data.public_key.q_y) * 8 + 4) / 5);
      let mut enc: [u8; LEN] = uninitialized();
      let res = ll::GNUNET_STRINGS_data_to_string(self.data.public_key.q_y.as_ptr() as *const c_void,
                                                  self.data.public_key.q_y.len() as size_t,
                                                  enc.as_mut_ptr() as *mut c_char,
                                                  enc.len() as size_t);
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

