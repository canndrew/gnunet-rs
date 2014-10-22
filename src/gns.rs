use std::io::{IoError, MemWriter};
use std::io::net::pipe::UnixStream;
use std::io::util::LimitReader;
use std::collections::HashMap;
use std::kinds::marker::InvariantLifetime;
use sync::comm::{Empty, Disconnected};

use identity;
use FromError;
use ll;
use service::{Service, ProcessMessageResult};
use service::{ServiceContinue, ServiceReconnect, ServiceShutdown};
use GNSRecord;
use ServiceConnectError;
use EcdsaPublicKey;
use EcdsaPrivateKey;
use GNSRecordType;
use IdentityService;
use Configuration;

pub struct GNS {
  service: Service,
  lookup_id: u32,
  lookup_tx: Sender<(u32, Sender<GNSRecord>)>,
}

pub enum LocalOptions {
  LODefault     = 0,
  LONoDHT       = 1,
  LOLocalMaster = 2,
}

#[deriving(Show)]
pub enum LookupError {
  NameTooLong,
  Io(IoError),
  ServiceConnect(ServiceConnectError),
  IdentityLookup(identity::GetDefaultError),
}

impl FromError<IoError> for LookupError {
  fn from_error(e: IoError) -> LookupError {
    Io(e)
  }
}

impl FromError<ServiceConnectError> for LookupError {
  fn from_error(e: ServiceConnectError) -> LookupError {
    ServiceConnect(e)
  }
}

impl FromError<identity::GetDefaultError> for LookupError {
  fn from_error(e: identity::GetDefaultError) -> LookupError {
    IdentityLookup(e)
  }
}

impl GNS {
  pub fn connect(cfg: Option<Configuration>) -> Result<GNS, ServiceConnectError> {
    let (lookup_tx, lookup_rx) = channel::<(u32, Sender<GNSRecord>)>();
    let mut handles: HashMap<u32, Sender<GNSRecord>> = HashMap::new();

    //let service = ttry!(Service::connect("gns", move |&mut: tpe: u16, reader: LimitReader<&mut Reader>| -> ProcessMessageResult {
    let service = ttry!(Service::connect_loop(cfg, "gns", move |&mut: tpe: u16, mut reader: LimitReader<UnixStream>| -> ProcessMessageResult {
      loop {
        match lookup_rx.try_recv() {
          Ok((id, sender)) => {
            handles.insert(id, sender);
          },
          Err(e)  => match e {
            Empty         => break,
            Disconnected  => return ServiceShutdown,
          },
        }
      }
      match tpe {
        ll::GNUNET_MESSAGE_TYPE_GNS_LOOKUP_RESULT => {
          let id = match reader.read_be_u32() {
            Ok(id)  => id,
            Err(_)  => return ServiceReconnect,
          };
          match handles.find(&id) {
            Some(sender) => {
              let rd_count = match reader.read_be_u32() {
                Ok(x)   => x,
                Err(_)  => return ServiceReconnect,
              };
              for _ in range(0, rd_count) {
                let rec = match GNSRecord::deserialize(&mut reader) {
                  Ok(r)   => r,
                  Err(_)  => return ServiceReconnect,
                };
                sender.send(rec);
              };
            },
            _ => (),
          };
        },
        _ => return ServiceReconnect,
      };
      match reader.limit() {
        0 => ServiceContinue,
        _ => ServiceReconnect,
      }
    }));
    Ok(GNS {
      service: service,
      lookup_id: 0,
      lookup_tx: lookup_tx,
    })
  }

  pub fn lookup_in_zone<'a>(
      &'a mut self,
      name: &str,
      zone: &EcdsaPublicKey,
      record_type: GNSRecordType,
      options: LocalOptions,
      shorten: Option<&EcdsaPrivateKey>) -> Result<LookupHandle<'a>, LookupError> {

    let name_len = name.len();
    if name_len > ll::GNUNET_DNSPARSER_MAX_NAME_LENGTH as uint {
      return Err(NameTooLong);
    };

    let id = self.lookup_id;
    self.lookup_id += 1;

    let msg_length = (80 + name_len + 1).to_u16().unwrap();
    let mut mw = MemWriter::with_capacity(msg_length as uint);

    ttry!(mw.write_be_u16(msg_length));
    ttry!(mw.write_be_u16(ll::GNUNET_MESSAGE_TYPE_GNS_LOOKUP));
    ttry!(mw.write_be_u32(id));
    ttry!(zone.serialize(&mut mw));
    ttry!(mw.write_be_i16(options as i16));
    ttry!(mw.write_be_i16(shorten.is_some() as i16));
    ttry!(mw.write_be_i32(record_type as i32));
    match shorten {
      Some(z) => ttry!(z.serialize(&mut mw)),
      None    => ttry!(mw.write([0u8, ..32])),
    };
    ttry!(mw.write(name.as_bytes()));
    ttry!(mw.write_u8(0u8));

    let v = mw.unwrap();
    assert!(v.len() == msg_length as uint);

    let (tx, rx) = channel::<GNSRecord>();
    self.lookup_tx.send((id, tx));
    ttry!(self.service.write(v[]));
    Ok(LookupHandle {
      marker: InvariantLifetime,
      receiver: rx,
    })
  }

  pub fn lookup<'a>(
      &'a mut self,
      name: &str,
      record_type: GNSRecordType,
      shorten: Option<&EcdsaPrivateKey>) -> Result<LookupHandle<'a>, LookupError> {
    let mut is = ttry!(IdentityService::connect(Some(self.service.cfg.clone())));
    let ego = ttry!(is.get_default_ego("gns-master"));
    let pk = ego.get_public_key();
    let mut it = name.split('.');
    let opt = match (it.next(), it.next(), it.next()) {
      (Some(_), Some("gnu"), None)  => LONoDHT,
      _                             => LOLocalMaster,
    };
    self.lookup_in_zone(name, &pk, record_type, opt, shorten)
  }
}

pub fn lookup_in_zone(
    cfg: Option<Configuration>,
    name: &str,
    zone: &EcdsaPublicKey,
    record_type: GNSRecordType,
    options: LocalOptions,
    shorten: Option<&EcdsaPrivateKey>) -> Result<GNSRecord, LookupError> {
  let mut gns = ttry!(GNS::connect(cfg));
  let mut h = ttry!(gns.lookup_in_zone(name, zone, record_type, options, shorten));
  Ok(h.recv())
}

pub fn lookup(
    cfg: Option<Configuration>,
    name: &str,
    record_type: GNSRecordType,
    shorten: Option<&EcdsaPrivateKey>) -> Result<GNSRecord, LookupError> {
  let mut gns = ttry!(GNS::connect(cfg));
  let mut h = ttry!(gns.lookup(name, record_type, shorten));
  Ok(h.recv())
}

pub struct LookupHandle<'a> {
  marker: InvariantLifetime<'a>,
  receiver: Receiver<GNSRecord>,
}

impl<'a> LookupHandle<'a> {
  pub fn recv(&mut self) -> GNSRecord {
    self.receiver.recv()
  }
}


