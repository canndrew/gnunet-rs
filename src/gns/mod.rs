use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num::ToPrimitive;
use std::collections::HashMap;
use std::io::{self, Cursor, Write};
use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

pub use self::record::*;
use identity;
use ll;
use service::{self, ProcessMessageResult, ServiceReadLoop, ServiceWriter};
use Cfg;
use EcdsaPrivateKey;
use EcdsaPublicKey;

mod record;

/// A handle to a locally-running instance of the GNS daemon.
pub struct GNS {
    service_writer: ServiceWriter,
    _callback_loop: ServiceReadLoop,
    lookup_id: u32,
    lookup_tx: Sender<(u32, Sender<Record>)>,
}

/// Options for GNS lookups.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LocalOptions {
    /// Default behaviour. Look in the local cache, then in the DHT.
    Default = 0,
    /// Do not look in the DHT, keep the request to the local cache.
    NoDHT = 1,
    /// For domains controlled by our master zone only look in the cache. Otherwise look in the
    /// cache, then in the DHT.
    LocalMaster = 2,
}

quick_error! {
    /// Possible errors returned by the GNS lookup functions.
    #[derive(Debug)]
    pub enum LookupError {
        NameTooLong { name: String } {
            display("The domain name was too long. The domain name \"{}\" is too long to lookup.", name)
        }

        Io { cause: io::Error } {
            cause(cause)
            from(cause: io::Error) -> { cause: cause }
            display("There was an I/O error communicating with the service. Specifically {}", cause)
        }
    }
}

impl GNS {
    /// Connect to the GNS service.
    ///
    /// Returns either a handle to the GNS service or a `service::ConnectError`. `cfg` contains the
    /// configuration to use to connect to the service.
    pub fn connect(cfg: &Cfg) -> Result<GNS, service::ConnectError> {
        let (lookup_tx, lookup_rx) = channel::<(u32, Sender<Record>)>();
        let mut handles: HashMap<u32, Sender<Record>> = HashMap::new();

        let (service_reader, service_writer) = try!(service::connect(cfg, "gns"));
        let callback_loop = try!(service_reader.spawn_callback_loop(
            move |tpe: u16, mut reader: Cursor<Vec<u8>>| -> ProcessMessageResult {
                println!("GNS got message!");
                loop {
                    match lookup_rx.try_recv() {
                        Ok((id, sender)) => {
                            handles.insert(id, sender);
                        }
                        Err(e) => match e {
                            TryRecvError::Empty => break,
                            TryRecvError::Disconnected => return ProcessMessageResult::Shutdown,
                        },
                    }
                }

                println!("tpe == {}", tpe);

                // TODO: drop expired senders, this currently leaks memory as `handles` only gets bigger
                //       need a way to detect when the remote Receiver has hung up
                match tpe {
                    ll::GNUNET_MESSAGE_TYPE_GNS_LOOKUP_RESULT => {
                        let id = match reader.read_u32::<BigEndian>() {
                            Ok(id) => id,
                            Err(_) => return ProcessMessageResult::Reconnect,
                        };
                        println!("WOW id == {}", id);
                        match handles.get(&id) {
                            Some(sender) => {
                                println!("WOW there's a sender for that");
                                let rd_count = match reader.read_u32::<BigEndian>() {
                                    Ok(x) => x,
                                    Err(_) => return ProcessMessageResult::Reconnect,
                                };
                                println!("WOW rd_count == {}", rd_count);
                                for _ in 0..rd_count {
                                    let rec = match Record::deserialize(&mut reader) {
                                        Ok(r) => r,
                                        Err(_) => return ProcessMessageResult::Reconnect,
                                    };
                                    println!("WOW we deserialised it");
                                    let _ = sender.send(rec);
                                }
                            }
                            _ => (),
                        };
                    }
                    _ => return ProcessMessageResult::Reconnect,
                };
                ProcessMessageResult::Continue
            }
        ));
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
    /// use gnunet::{Cfg, IdentityService, GNS, gns};
    ///
    /// let config = Cfg::default().unwrap();
    /// let mut ids = IdentityService::connect(&config).unwrap();
    /// let gns_ego = ids.get_default_ego("gns-master").unwrap();
    /// let mut gns = GNS::connect(&config).unwrap();
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
        shorten: Option<&EcdsaPrivateKey>,
    ) -> Result<LookupHandle<'a>, LookupError> {
        let name_len = name.len();
        if name_len > ll::GNUNET_DNSPARSER_MAX_NAME_LENGTH as usize {
            return Err(LookupError::NameTooLong {
                name: name.to_string(),
            });
        };

        let id = self.lookup_id;
        self.lookup_id += 1;

        let msg_length = (80 + name_len + 1).to_u16().unwrap();
        let mut mw = self
            .service_writer
            .write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_GNS_LOOKUP);
        mw.write_u32::<BigEndian>(id).unwrap();
        zone.serialize(&mut mw).unwrap();
        mw.write_i16::<BigEndian>(options as i16).unwrap();
        mw.write_i16::<BigEndian>(shorten.is_some() as i16).unwrap();
        mw.write_i32::<BigEndian>(record_type as i32).unwrap();
        match shorten {
            Some(z) => z.serialize(&mut mw).unwrap(),
            None => mw.write_all(&[0u8; 32]).unwrap(),
        };
        mw.write_all(name.as_bytes()).unwrap();
        mw.write_u8(0u8).unwrap();

        let (tx, rx) = channel::<Record>();
        self.lookup_tx.send((id, tx)).unwrap(); // panics if the callback loop has panicked
        try!(mw.send());
        Ok(LookupHandle {
            marker: PhantomData,
            receiver: rx,
        })
    }
}

quick_error! {
    /// Errors returned by `gns::lookup`.
    #[derive(Debug)]
    pub enum ConnectLookupError {
        Connect { cause: service::ConnectError } {
            display("Failed to connect to the GNS service. Reason: {}", cause)
            cause(cause)
            from(cause: service::ConnectError) -> { cause: cause }
        }

        Lookup { cause: LookupError } {
            display("Failed to perform the lookup. Reason: {}", cause)
            cause(cause)
            from(cause: LookupError) -> { cause: cause }
        }
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
/// use gnunet::{Cfg, identity, gns};
///
/// let config = Cfg::default().unwrap();
/// let gns_ego = identity::get_default_ego(&config, "gns-master").unwrap();
/// let record = gns::lookup(&config,
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
    cfg: &Cfg,
    name: &str,
    zone: &EcdsaPublicKey,
    record_type: RecordType,
    options: LocalOptions,
    shorten: Option<&EcdsaPrivateKey>,
) -> Result<Record, ConnectLookupError> {
    println!("connecting to GNS");
    let mut gns = try!(GNS::connect(cfg));
    println!("connected to GNS");
    let mut h = try!(gns.lookup(name, zone, record_type, options, shorten));
    println!("doing lookup");
    Ok(h.recv())
}

quick_error! {
    /// Errors returned by `gns::lookup_in_master`.
    #[derive(Debug)]
    pub enum ConnectLookupInMasterError {
        GnsLookup { cause: ConnectLookupError } {
            cause(cause)
            from(cause :ConnectLookupError) -> { cause: cause }
            display("Failed to connect to the GNS service and perform the lookup. Reason: {}", cause)
        }

        IdentityGetDefaultEgo { cause: identity::ConnectGetDefaultEgoError } {
            cause(cause)
            from(cause: identity::ConnectGetDefaultEgoError) -> { cause: cause }
            display("Failed to retrieve the default identity for gns-master from the identity service. Reason: {}", cause)
        }
    }
}

/// Lookup a GNS record in the master zone.
///
/// If `shorten` is not `None` then the result is added to the given shorten zone. This function
/// will block until it returns the first matching record that it can find.
///
/// # Example
///
/// ```rust
/// use gnunet::{Cfg, gns};
///
/// println!("in test lookup_in_master");
///
/// let config = Cfg::default().unwrap();
/// let record = gns::lookup_in_master(&config, "www.gnu", gns::RecordType::A, None).unwrap();
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
    cfg: &Cfg,
    name: &str,
    record_type: RecordType,
    shorten: Option<&EcdsaPrivateKey>,
) -> Result<Record, ConnectLookupInMasterError> {
    println!("Getting default ego");
    let ego = try!(identity::get_default_ego(cfg, "gns-master"));
    println!("got default ego: {}", ego);
    let pk = ego.get_public_key();
    let mut it = name.split('.');
    let opt = match (it.next(), it.next(), it.next()) {
        (Some(_), Some("gnu"), None) => LocalOptions::NoDHT,
        _ => LocalOptions::LocalMaster,
    };
    println!("doing lookup");
    let ret = try!(lookup(cfg, name, &pk, record_type, opt, shorten));
    println!("lookup succeeded");
    Ok(ret)
}

/// A handle returned by `GNS::lookup`.
///
/// Used to retrieve the results of a lookup.
pub struct LookupHandle<'a> {
    marker: PhantomData<&'a GNS>,
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
