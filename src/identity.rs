use std::io::{Reader, IoError};
use std::io::util::LimitReader;
use std::str::from_utf8;
use std::io::MemWriter;
use std::collections::HashMap;

use ll;
use EcdsaPrivateKey;
use EcdsaPublicKey;
use HashCode;
use ServiceConnectError;
use service;
use service::Service;
use FromError;
use Configuration;

/// A GNUnet identity.
///
/// An ego consists of a public/private key pair and a name.
#[deriving(Clone)]
pub struct Ego {
  pk: EcdsaPrivateKey,
  name: Option<String>,
  id: HashCode,
}

impl Ego {
  /// Get a copy of the global, anonymous ego.
  pub fn anonymous() -> Ego {
    let pk = EcdsaPrivateKey::anonymous();
    Ego {
      pk: pk,
      name: None,
      id: pk.get_public().hash(),
    }
  }

  /// Get the public key of an ego.
  pub fn get_public_key(&self) -> EcdsaPublicKey {
    self.pk.get_public()
  }

  /// Get the private key of an ego.
  pub fn get_private_key(&self) -> EcdsaPrivateKey {
    self.pk
  }

  /// Get the name of an ego.
  pub fn get_name(&self) -> Option<String> {
    self.name.clone()
  }

  /// Get the unique id of an ego. This is a hash of the ego's public key.
  pub fn get_id(&self) -> HashCode {
    self.id
  }
}

/// Errors returned by `get_default_ego`. 
#[deriving(Show)]
pub enum GetDefaultError {
  /// The name of the service was too long.
  NameTooLong,
  /// An I/O error occured while communicating with the identity service.
  Io(IoError),
  /// The service responded with an error message.
  ServiceResponse(String),
  /// Failed to connect to the identity service.
  ServiceConnect(ServiceConnectError),
  /// The service response was incoherent. You should file a bug-report if you encounter this
  /// variant.
  InvalidResponse,
}
error_chain!(ServiceConnectError, GetDefaultError, ServiceConnect)
error_chain!(IoError, GetDefaultError, Io)

/// A handle to the identity service.
pub struct IdentityService {
  service: Service,
  egos: HashMap<HashCode, Ego>,
}

impl IdentityService {
  /// Connect to the identity service.
  ///
  /// Returns either a handle to the identity service or a `ServiceConnectError`. `cfg` contains
  /// the configuration to use to connect to the service. Can be `None` to use the system default
  /// configuration - this should work on most properly-configured systems.
  pub fn connect(cfg: Option<Configuration>) -> Result<IdentityService, ServiceConnectError> {
    /*
    let (get_tx, get_rx) = channel::<(String, Sender<Option<Ego>>>();
    let service = ttry!(Service::connect("identity", move |&mut: tpe: u16, mut reader: LimitReader<UnixStream>| -> ProcessMessageResult {
      loop {
        
      }
    }));
    */
    let service = ttry!(Service::connect(cfg, "identity"));
    let mut stream = (*service.connection).clone();
    let mut ret = IdentityService {
      service: service,
      egos: HashMap::new(),
    };
    ttry!(stream.write_be_u16(4));
    ttry!(stream.write_be_u16(ll::GNUNET_MESSAGE_TYPE_IDENTITY_START));
    loop {
      let msg_length = ttry!(stream.read_be_u16());
      let tpe = ttry!(stream.read_be_u16());
      let mut lr = LimitReader::new(stream.clone(), (msg_length - 4) as uint);
      match tpe {
        ll::GNUNET_MESSAGE_TYPE_IDENTITY_UPDATE => {
          let name_len = ttry!(lr.read_be_u16());
          let eol = ttry!(lr.read_be_u16());
          if eol != 0 {
            for _ in range(0, lr.limit()) {
              ttry!(lr.read_u8());
            }
            break;
          };
          let pk = ttry!(EcdsaPrivateKey::deserialize(&mut lr));
          let mut v: Vec<u8> = Vec::with_capacity(name_len as uint);
          for r in lr.bytes() {
            let b = ttry!(r);
            if b == 0u8 {
              break;
            }
            v.push(b)
          };
          let name = match String::from_utf8(v) {
            Ok(n)   => n,
            Err(_)  => return Err(service::InvalidResponse),
          };
          let id = pk.get_public().hash();
          ret.egos.insert(id, Ego {
            pk: pk,
            name: Some(name),
            id: id,
          });
        },
        _ => return Err(service::InvalidResponse),
      };
    };
    Ok(ret)
  }

  /// Get the default identity associated with a service.
  ///
  /// # Example
  ///
  /// Get the ego for the default master zone.
  ///
  /// ```rust
  /// use gnunet::IdentityService;
  ///
  /// let mut ids = IdentityService::connect(None);
  /// let ego = ids.get_default_ego(None, "gns-master").unwrap();
  /// ```
  pub fn get_default_ego(&mut self, name: &str) -> Result<Ego, GetDefaultError> {
    let name_len = name.len();

    let msg_length = match (8 + name_len + 1).to_u16() {
      Some(l) => l,
      None    => return Err(NameTooLong),
    };
    let mut mw = MemWriter::with_capacity(msg_length as uint);

    ttry!(mw.write_be_u16(msg_length));
    ttry!(mw.write_be_u16(ll::GNUNET_MESSAGE_TYPE_IDENTITY_GET_DEFAULT));
    ttry!(mw.write_be_u16((name_len + 1) as u16));
    ttry!(mw.write_be_u16(0));
    ttry!(mw.write(name.as_bytes()));
    ttry!(mw.write_u8(0u8));

    let v = mw.unwrap();
    assert!(v.len() == msg_length as uint);

    ttry!(self.service.write(v[]));
    
    let reply_len = ttry!(self.service.read_be_u16());
    let tpe = ttry!(self.service.read_be_u16());
    let mut lr = LimitReader::new((*self.service.connection).clone(), (reply_len - 4) as uint);
    let ret = match tpe {
      ll::GNUNET_MESSAGE_TYPE_IDENTITY_RESULT_CODE  => {
        ttry!(lr.read_be_u32());
        let mut v: Vec<u8> = Vec::new();
        for r in lr.bytes() {
          let b = ttry!(r);
          if b == 0u8 {
            break;
          }
          v.push(b)
        };
        match String::from_utf8(v) {
          Ok(s)   => Err(ServiceResponse(s)),
          Err(_)  => Err(InvalidResponse),
        }
      },
      ll::GNUNET_MESSAGE_TYPE_IDENTITY_SET_DEFAULT => {
        let reply_name_len = ttry!(lr.read_be_u16());
        let zero = ttry!(lr.read_be_u16());
        match zero {
          0 => {
            let pk = ttry!(EcdsaPrivateKey::deserialize(&mut lr));
            let mut v: Vec<u8> = Vec::with_capacity(reply_name_len as uint);
            for r in lr.bytes() {
              let b = ttry!(r);
              if b == 0u8 {
                break;
              }
              v.push(b)
            };
            match String::from_utf8(v) {
              Ok(s)   => match s[] == name {
                true  =>  {
                  let id = pk.get_public().hash();
                  Ok(self.egos[id].clone())
                },
                false => Err(InvalidResponse),
              },
              Err(_)  => Err(InvalidResponse),
            }
          },
          _ => Err(InvalidResponse),
        }
      },
      _ => Err(InvalidResponse),
    };
    assert!(lr.limit() == 0);
    ret
  }
}

/// Get the default identity associated with a service.
///
/// # Example
///
/// Get the ego for the default master zone.
///
/// ```rust
/// use gnunet::identity;
///
/// let ego = identity::get_default_ego(None, "gns-master").unwrap();
/// ```
///
/// # Note
///
/// This a convenience function that connects to the identity service, does the query, then
/// disconnects. If you want to do multiple queries you should connect to the service with
/// `IdentityService::connect` then use that handle to do the queries.
pub fn get_default_ego(
    cfg: Option<Configuration>,
    name: &str) -> Result<Ego, GetDefaultError> {
  let mut is = ttry!(IdentityService::connect(cfg));
  is.get_default_ego(name)
}

