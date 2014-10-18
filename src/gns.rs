use std::io::{Stream, IoResult, IoError, MemWriter};
use std::io::net::pipe::UnixStream;
use std::io::util::LimitReader;
use std::collections::HashMap;
use std::mem::uninitialized;
use libc::c_void;

use FromError;
use ll;
use service::{Service, ProcessMessageResult, Continue, Reconnect, Shutdown};
use Configuration;
use GNSRecord;
use ServiceConnectError;
use EcdsaPublicKey;
use EcdsaPrivateKey;
use GNSRecordType;

pub struct GNS {
  service: Service,
  lookup_id: u32,
  lookup_tx: Sender<(u32, Sender<GNSRecord>)>,
}

pub enum LocalOptions {
  LODefault     = 0,
  LONoDHT       = 1,
  NOLocalMaster = 2,
}

pub struct LookupHandle<'a> {
  gns: &'a GNS,
  id: u32,
  receiver: Receiver<GNSRecord>,
}

pub enum LookupError {
  NameTooLong,
  Io(IoError),
}

impl FromError<IoError> for LookupError {
  fn from_error(e: IoError) -> LookupError {
    Io(e)
  }
}

impl GNS {
  pub fn connect() -> Result<GNS, ServiceConnectError> {
    let (lookup_tx, lookup_rx) = channel::<(u32, Sender<GNSRecord>)>();
    let mut handles: HashMap<u32, Sender<GNSRecord>> = HashMap::new();

    let service = ttry!(Service::connect("gns", move |&mut: tpe: u16, reader: LimitReader<&mut Reader>| -> ProcessMessageResult {
      loop {
        match lookup_rx.try_recv() {
          Ok((id, sender)) => {
            handles.insert(id, sender);
          },
          Err(e)  => match e {
            Empty         => break,
            //Disconnected  => return Shutdown,
          },
        }
      }
      match tpe {
        ll::GNUNET_MESSAGE_TYPE_GNS_LOOKUP_RESULT => {
          let id = match reader.read_be_u32() {
            Ok(id)  => id,
            Err(_)  => return Reconnect,
          };
          match handles.find(&id) {
            Some(sender) => {
              let rd_count = match reader.read_be_u32() {
                Ok(x)   => x,
                Err(_)  => return Reconnect,
              };
              for _ in range(0, rd_count) {
                let rec = match GNSRecord::deserialize(&reader) {
                  Ok(r)   => r,
                  Err(_)  => return Reconnect,
                };
                sender.send(rec);
              };
            },
            _ => (),
          };
        },
        _ => return Reconnect,
      };
      match reader.limit() {
        0 => Continue,
        _ => Reconnect,
      }
    }));
    Ok(GNS {
      service: service,
      lookup_id: 0,
      lookup_tx: lookup_tx,
    })
  }

  pub fn lookup(
      &mut self,
      name: &str,
      zone: &EcdsaPublicKey,
      record_type: GNSRecordType,
      options: LocalOptions,
      shorten: Option<&EcdsaPrivateKey>) -> Result<LookupHandle, LookupError> {

    let name_len = name.as_bytes().len();
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
    ttry!(zone.serialize(&mw));
    ttry!(mw.write_be_i16(options as i16));
    ttry!(mw.write_be_i16(shorten.is_some() as i16));
    ttry!(mw.write_be_i32(record_type as i32));
    match shorten {
      Some(z) => ttry!(z.serialize(&mw)),
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
      gns: self,
      id: id,
      receiver: rx,
    })
  }
}

