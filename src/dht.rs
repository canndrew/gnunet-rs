use std::old_io::net::pipe::UnixStream;
use std::old_io::util::LimitReader;

use Configuration;
use service::{Service, ConnectError, ProcessMessageResult};

pub enum BlockType {
  /// Any type of block, used as a wildcard when searching. Should never be attached to a specific
  /// block.
  Any = 0,

  /// Data block (leaf) in the CHK tree.
  FsDBlock = 1,

  /// Inner block in the CHK tree.
  FsIBlock = 2,

  /// Legacy type, no longer in use.
  FsKBLock = 3,

  /// Legacy type, no longer in use.
  FsSBLock = 4,

  /// Legacy type, no longer in use.
  FsNBlock = 5,

  /// Type of a block representing a block to be encoded on demand from disk. Should never appear
  /// on the network directly.
  FsOnDemand = 6,

  /// Type of a block that contains a HELLO for a peer (for
  /// DHT and CADET find-peer operations).
  DHTHello = 7,

  /// Block for testing.
  Test = 8,

  /// Type of a block representing any type of search result (universal). Implemented in the
  /// context of GNUnet bug #2564, replaces SBLOCKS, KBLOCKS and NBLOCKS.
  FsUBlock = 9,

  /// Block for storing DNS exit service advertisements.
  DNS = 10,

  /// Block for storing record data.
  GNSNameRecord = 11,

  /// Block to store a cadet regex state.
  Regex = 22,

  /// Block to store a cadet regex accepting state.
  RegexAccept = 23
}

struct RouteOptions {
  bits: u32,
}

impl RouteOptions {
  pub static DEMULTIPLEX_EVERYWHERE: u32 = 1;
  pub static RECORD_ROUTE: u32 = 2;
  pub static FIND_PEER: u32 = 4;
  pub static BART: u32 = 8;
  pub static LAST_HOP: u32 = 16;

  #[inline]
  pub fn demultiplex_everywhere(&self) -> bool {
    0 != (self.bits & DEMULTIPLEX_EVERYWHERE)
  }

  #[inline]
  pub fn record_route(&self) -> bool {
    0 != (self.bits & RECORD_ROUTE)
  }

  #[inline]
  pub fn find_peer(&self) -> bool {
    0 != (self.bits & FIND_PEER)
  }

  #[inline]
  pub fn bart(&self) -> bool {
    0 != (self.bits & BART)
  }

  #[inline]
  pub fn last_hop(&self) -> bool {
    0 != (self.bits & LAST_HOP)
  }
}

struct GetResult {
  expires: Tm,
  key: HashCode,
  get_path: Option<Vec<PeerIdentity>>;
  put_path: Option<Vec<PeerIdentity>>;
  block_type: u32,
  data: Vec<u8>,
}

struct GetGnsNameRecordResult {
  expires: Tm,
  get_path: Option<Vec<PeerIdentity>>;
  put_path: Option<Vec<PeerIdentity>>;
  data: Vec<u8>,
}

struct GetHandle<'a> {
  marker: InvariantLifetime<'a>,
  receiver: Receiver<GetResult>,
}

struct GetGnsNameRecordHandle<'a> {
  marker: InvariantLifetime<'a>,
  receiver: Receiver<GetGnsNameRecordResult>,
}

pub struct DHT {
  service: Service,
  next_get_id: u64,
}

impl DHT {
  pub fn connect(cfg: Option<&Configuration>) -> Result<DHT, ConnectError> {
    let mut service = ttry!(Service::connect(cfg, "dht"));
    service.init_callback_loop(move |&mut: tpe: u16, mut read: LimitReader<UnixStream>| -> ProcessMessageResult {
      ProcessMessageResult::Continue
    });
    Ok(DHT {
      service: service,
      next_get_id: 1,
    })
  }

  pub fn get_gns_name_record<'a>(
      &'a mut self,
      key: &HashCode,
      desired_replication_level: u32,
      route_options: RouteOptions) {
    let gh = self.get(BlockType::GNSNameRecord as u32,
                      key,
                      desired_replication_level,
                      route_options,
                      &[]);

    let check_key = key.clone();
    let (tx, rx) = channel::<GetGnsNameRecordResult>();
    spawn(move |:| {
      loop {
        let pull = try!(gh.receiver.recv_opt());
        if pull.key != check_key {
          continue;
        }
        if pull.block_type != BlockType::GNSNameRecord as u32 {
          continue;
        }
        let push = GetGnsNameRecordResult {
          expires: pull.expires,
          get_path: pull.get_path,
          put_path: pull.put_path,
          data: pull.data,
        }
        try!(tx.send_opt(push).map_err(|_| ()));
      }
    });
    Ok(GetGnsNameRecordHandle {
      marker: InvariantLifetime,
      receiver: rx,
    })
  }

  pub fn get<'a>(
      &'a mut self,
      block_type: u32,
      key: &HashCode,
      desired_replication_level: u32,
      route_options: RouteOptions,
      xquery: &[u8])
  {
    let msg_length = 88 + xquery.len();
    let mut mw = self.service.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_DHT_CLIENT_GET);
    ttry!(mw.write_be_u32(route_options.bits));
    ttry!(mw.write_be_u32(desired_replication_level));
    ttry!(mw.write_be_u32(block_type));
    ttry!(key.serialize(mw));
    let id = self.next_get_id;
    ttry!(mw.write_be_u64(id));
    self.next_get_id += 1;
    let (tx, rx) = channel::<GetResult>();
    self.lookup_tx.send((id, tx));
    ttry!(mw.send());
    Ok(GetHandle {
      marker: InvariantLifetime,
      receiver: rx,
    })
  }
}

