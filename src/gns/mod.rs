use std::io::net::pipe::UnixStream;
use std::io::util::LimitReader;
use std::collections::HashMap;
use std::marker::InvariantLifetime;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::sync::Arc;
use std::num::ToPrimitive;

use identity;
use ll;
use service::{self, ServiceReadLoop, ServiceWriter, ProcessMessageResult};
use EcdsaPublicKey;
use EcdsaPrivateKey;
use Configuration;
use self::error::*;
pub use self::record::*;

mod error;
mod record;

/// A handle to a locally-running instance of the GNS daemon.
pub struct GNS {
  service_writer: ServiceWriter,
  _callback_loop: ServiceReadLoop,
  lookup_id: u32,
  lookup_tx: Sender<(u32, Sender<Record>)>,
}

/// Options for GNS lookups.
#[derive(Copy, Clone, Show, PartialEq, Eq)]
pub enum LocalOptions {
  /// Default behaviour. Look in the local cache, then in the DHT.
  Default     = 0,
  /// Do not look in the DHT, keep the request to the local cache.
  NoDHT       = 1,
  /// For domains controlled by our master zone only look in the cache. Otherwise look in the
  /// cache, then in the DHT.
  LocalMaster = 2,
}

impl GNS {
  /// Connect to the GNS service.
  ///
  /// Returns either a handle to the GNS service or a `service::ConnectError`. `cfg` contains the
  /// configuration to use to connect to the service.
  pub fn connect(cfg: Arc<Configuration>) -> Result<GNS, service::ConnectError> {
    let (lookup_tx, lookup_rx) = channel::<(u32, Sender<Record>)>();
    let mut handles: HashMap<u32, Sender<Record>> = HashMap::new();

    let (service_reader, service_writer) = try!(service::connect(cfg, "gns"));
    let callback_loop = service_reader.spawn_callback_loop(move |&mut: tpe: u16, mut reader: LimitReader<UnixStream>| -> ProcessMessageResult {
      loop {
        match lookup_rx.try_recv() {
          Ok((id, sender)) => {
            handles.insert(id, sender);
          },
          Err(e)  => match e {
            TryRecvError::Empty         => break,
            TryRecvError::Disconnected  => return ProcessMessageResult::Shutdown,
          },
        }
      }
      // TODO: drop expired senders, this currently leaks memory as `handles` only gets bigger
      //       need a way to detect when the remote Receiver has hung up
      match tpe {
        ll::GNUNET_MESSAGE_TYPE_GNS_LOOKUP_RESULT => {
          let id = match reader.read_be_u32() {
            Ok(id)  => id,
            Err(_)  => return ProcessMessageResult::Reconnect,
          };
          match handles.get(&id) {
            Some(sender) => {
              let rd_count = match reader.read_be_u32() {
                Ok(x)   => x,
                Err(_)  => return ProcessMessageResult::Reconnect,
              };
              for _ in range(0, rd_count) {
                let rec = match Record::deserialize(&mut reader) {
                  Ok(r)   => r,
                  Err(_)  => return ProcessMessageResult::Reconnect,
                };
                let _ = sender.send(rec);
              };
            },
            _ => (),
          };
        },
        _ => return ProcessMessageResult::Reconnect,
      };
      match reader.limit() {
        0 => ProcessMessageResult::Continue,
        _ => ProcessMessageResult::Reconnect,
      }
    });
    Ok(GNS {
      service_writer: service_writer,
      _callback_loop: callback_loop,
      lookup_id: 0,
      lookup_tx: lookup_tx,
    })
  }

  /// Lookup a GNS record in the given zone.
  ///
  /// If `shorten` is not `None` then the result is added to the given shorten zone. Returns
  /// immediately with a handle that can be queried for results.
  ///
  /// # Example
  ///
  /// ```rust
  /// use std::sync::Arc;
  /// use gnunet::{Configuration, IdentityService, GNS, gns};
  ///
  /// let config = Arc::new(Configuration::default().unwrap());
  /// let mut ids = IdentityService::connect(config.clone()).unwrap();
  /// let gns_ego = ids.get_default_ego("gns-master").unwrap();
  /// let mut gns = GNS::connect(config).unwrap();
  /// let mut lh = gns.lookup("www.gnu",
  ///                         &gns_ego.get_public_key(),
  ///                         gns::RecordType::A,
  ///                         gns::LocalOptions::LocalMaster,
  ///                         None).unwrap();
  /// let record = lh.recv();
  /// println!("Got the IPv4 record for www.gnu: {}", record);
  /// ```
  pub fn lookup<'a>(
      &'a mut self,
      name: &str,
      zone: &EcdsaPublicKey,
      record_type: RecordType,
      options: LocalOptions,
      shorten: Option<&EcdsaPrivateKey>
    ) -> Result<LookupHandle<'a>, LookupError> {

    let name_len = name.len();
    if name_len > ll::GNUNET_DNSPARSER_MAX_NAME_LENGTH as usize {
      return Err(LookupError::NameTooLong);
    };

    let id = self.lookup_id;
    self.lookup_id += 1;

    let msg_length = (80 + name_len + 1).to_u16().unwrap();
    let mut mw = self.service_writer.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_GNS_LOOKUP);
    try!(mw.write_be_u32(id));
    try!(zone.serialize(&mut mw));
    try!(mw.write_be_i16(options as i16));
    try!(mw.write_be_i16(shorten.is_some() as i16));
    try!(mw.write_be_i32(record_type as i32));
    match shorten {
      Some(z) => try!(z.serialize(&mut mw)),
      None    => try!(mw.write(&[0u8; 32])),
    };
    try!(mw.write(name.as_bytes()));
    try!(mw.write_u8(0u8));

    let (tx, rx) = channel::<Record>();
    self.lookup_tx.send((id, tx)).unwrap(); // panics if the callback loop has panicked
    try!(mw.send());
    Ok(LookupHandle {
      marker: InvariantLifetime,
      receiver: rx,
    })
  }
}

/// Lookup a GNS record in the given zone.
///
/// If `shorten` is not `None` then the result is added to the given shorten zone. This function
/// will block until it returns the first matching record that it can find.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use gnunet::{Configuration, identity, gns};
///
/// let config = Arc::new(Configuration::default().unwrap());
/// let gns_ego = identity::get_default_ego(config.clone(), "gns-master").unwrap();
/// let record = gns::lookup(config,
///                          "www.gnu",
///                          &gns_ego.get_public_key(),
///                          gns::RecordType::A,
///                          gns::LocalOptions::LocalMaster,
///                          None).unwrap();
/// println!("Got the IPv4 record for www.gnu: {}", record);
/// ```
///
/// # Note
///
/// This is a convenience function that connects to the GNS service, performs the lookup, retrieves
/// one result, then disconects. If you are performing multiple lookups this function should be
/// avoided and `GNS::lookup_in_zone` used instead.
pub fn lookup(
    cfg: Arc<Configuration>,
    name: &str,
    zone: &EcdsaPublicKey,
    record_type: RecordType,
    options: LocalOptions,
    shorten: Option<&EcdsaPrivateKey>) -> Result<Record, ConnectLookupError> {
  let mut gns = try!(GNS::connect(cfg));
  let mut h = try!(gns.lookup(name, zone, record_type, options, shorten));
  Ok(h.recv())
}

/// Lookup a GNS record in the master zone.
///
/// If `shorten` is not `None` then the result is added to the given shorten zone. This function
/// will block until it returns the first matching record that it can find.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use gnunet::{Configuration, gns};
///
/// let config = Arc::new(Configuration::default().unwrap());
/// let record = gns::lookup_in_master(config, "www.gnu", gns::RecordType::A, None).unwrap();
/// println!("Got the IPv4 record for www.gnu: {}", record);
/// ```
///
/// # Note
///
/// This is a convenience function that connects to the identity service, fetches the default ego
/// for gns-master, then connects to the GNS service, performs the lookup, retrieves one result,
/// then disconnects from everything. If you are performing lots of lookups this function should be
/// avoided and `GNS::lookup_in_zone` used instead.
pub fn lookup_in_master(
    cfg: Arc<Configuration>,
    name: &str,
    record_type: RecordType,
    shorten: Option<&EcdsaPrivateKey>) -> Result<Record, ConnectLookupInMasterError> {
  let ego = try!(identity::get_default_ego(cfg.clone(), "gns-master"));
  let pk = ego.get_public_key();
  let mut it = name.split('.');
  let opt = match (it.next(), it.next(), it.next()) {
    (Some(_), Some("gnu"), None)  => LocalOptions::NoDHT,
    _                             => LocalOptions::LocalMaster,
  };
  let ret = try!(lookup(cfg, name, &pk, record_type, opt, shorten));
  Ok(ret)
}

/// A handle returned by `GNS::lookup`.
///
/// Used to retrieve the results of a lookup.
pub struct LookupHandle<'a> {
  marker: InvariantLifetime<'a>,
  receiver: Receiver<Record>,
}

impl<'a> LookupHandle<'a> {
  /// Receive a single result from a lookup.
  ///
  /// Blocks until a result is available. This function can be called multiple times on a handle to
  /// receive multiple results.
  pub fn recv(&mut self) -> Record {
    // unwrap is safe because the LookupHandle cannot outlive the remote sender.
    self.receiver.recv().unwrap()
  }
}

