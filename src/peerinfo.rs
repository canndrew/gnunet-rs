struct PeerIdentity {
  data: ll::Struct_GNUNET_PeerIdentity;
}

impl PeerIdentity {
  pub fn deserialize<T>(r: &mut T) -> IoResult<PeerIdentity> where T: Reader {
    let mut ret: PeerIdentity = unsafe { uninitialized() };
    ttry!(r.read(ret.data.public_key.q_y));
    Ok(ret)
  }
}

pub fn iterate_peers(cfg: Option<&Configuration>) -> PeerIterator {
  let mut service = ttry!(Service::connect(cfg, "peerinfo"));
  
  let msg_length = 8u16;
  let mut mw = ttry!(service.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_PEERINFO_GET_ALL));
  ttry!(mw.write_be_u32(0));
  ttry!(mw.send());
} 

struct PeerIterator {
  service: Service,
}

impl Iterator<Result<(PeerIdentity, Option<Hello>), WUB>> for PeerIterator {
  fn next(&mut self) -> Option<Result<(PeerIdentity, Option<Hello>), WUB>> {
    let (tpe, mut mr) = ttry!(self.service.read_message());
    match tpe {
      ll::GNUNET_MESSAGE_TYPE_PEERINFO_INFO => {
        if ttry!(mr.read_be_u32()) != 0 {
          return Some(Err(InvalidResponse)),
        };
        let pi = ttry!(PeerIdentity::deserialize(mr));
        let r_len = mr.read_be_u16();
        let r_tpe = mr.read_be_u16();
        match (r_len, r_tpe) {
          (Some(len), Some(tpe))  => {

          },
        }
      },
      ll::GNUNET_MESSAGE_TYPE_PEERINFO_INFO_END =>
      _ =>
    }
  }
}


