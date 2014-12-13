use std::io::{Reader, BytesReader};
use std::str::from_utf8;
use std::collections::HashMap;

use ll;
use util::CStringReader;
use EcdsaPrivateKey;
use EcdsaPublicKey;
use HashCode;
use service::Service;
use Configuration;
pub use self::error::*;

mod error;

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
    let id = pk.get_public().hash();
    Ego {
      pk: pk,
      name: None,
      id: id,
    }
  }

  /// Get the public key of an ego.
  pub fn get_public_key(&self) -> EcdsaPublicKey {
    self.pk.get_public()
  }

  /// Get the private key of an ego.
  pub fn get_private_key(&self) -> EcdsaPrivateKey {
    self.pk.clone()
  }

  /// Get the name of an ego.
  pub fn get_name(&self) -> Option<String> {
    self.name.clone()
  }

  /// Get the unique id of an ego. This is a hash of the ego's public key.
  pub fn get_id(&self) -> &HashCode {
    &self.id
  }
}

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
  pub fn connect(cfg: Option<&Configuration>) -> Result<IdentityService, ConnectError> {
    /*
    let (get_tx, get_rx) = channel::<(String, Sender<Option<Ego>>>();
    let service = ttry!(Service::connect("identity", move |&mut: tpe: u16, mut reader: LimitReader<UnixStream>| -> ProcessMessageResult {
      loop {
        
      }
    }));
    */
    let mut service = ttry!(Service::connect(cfg, "identity"));
    let mut egos: HashMap<HashCode, Ego> = HashMap::new();
    {
      let mw = service.write_message(4, ll::GNUNET_MESSAGE_TYPE_IDENTITY_START);
      ttry!(mw.send());
    };
    loop {
      let (tpe, mut mr) = ttry!(service.read_message());
      match tpe {
        ll::GNUNET_MESSAGE_TYPE_IDENTITY_UPDATE => {
          let name_len = ttry!(mr.read_be_u16());
          let eol = ttry!(mr.read_be_u16());
          if eol != 0 {
            break;
          };
          let pk = ttry!(EcdsaPrivateKey::deserialize(&mut mr));
          let mut v: Vec<u8> = Vec::with_capacity(name_len as uint);
          for r in mr.bytes() {
            let b = ttry!(r);
            if b == 0u8 {
              break;
            }
            v.push(b)
          };
          let name = match String::from_utf8(v) {
            Ok(n)   => n,
            Err(v)  => return Err(ConnectError::InvalidName(v)),
          };
          let id = pk.get_public().hash();
          egos.insert(id.clone(), Ego {
            pk: pk,
            name: Some(name),
            id: id,
          });
        },
        _ => return Err(ConnectError::UnexpectedMessageType(tpe)),
      };
    };
    Ok(IdentityService {
      service: service,
      egos: egos,
    })
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
  /// let mut ids = IdentityService::connect(None).unwrap();
  /// let ego = ids.get_default_ego("gns-master").unwrap();
  /// ```
  pub fn get_default_ego(&mut self, name: &str) -> Result<Ego, GetDefaultEgoError> {
    let name_len = name.len();

    let msg_length = match (8 + name_len + 1).to_u16() {
      Some(l) => l,
      None    => return Err(GetDefaultEgoError::NameTooLong),
    };
    {
      let mut mw = self.service.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_IDENTITY_GET_DEFAULT);
      ttry!(mw.write_be_u16((name_len + 1) as u16));
      ttry!(mw.write_be_u16(0));
      ttry!(mw.write(name.as_bytes()));
      ttry!(mw.write_u8(0u8));
      ttry!(mw.send());
    };

    let (tpe, mut mr) = ttry!(self.service.read_message());
    match tpe {
      ll::GNUNET_MESSAGE_TYPE_IDENTITY_RESULT_CODE => {
        ttry!(mr.read_be_u32());
        let s = ttry!(mr.read_cstring(None));
        Err(GetDefaultEgoError::ServiceResponse(s))
      },
      ll::GNUNET_MESSAGE_TYPE_IDENTITY_SET_DEFAULT => {
        let reply_name_len = ttry!(mr.read_be_u16());
        let zero = ttry!(mr.read_be_u16());
        match zero {
          0 => {
            let pk = ttry!(EcdsaPrivateKey::deserialize(&mut mr));
            let s = ttry!(mr.read_cstring(Some(reply_name_len as uint)));
            match s[] == name {
              true  =>  {
                let id = pk.get_public().hash();
                Ok(self.egos[id].clone())
              },
              false => Err(GetDefaultEgoError::InvalidResponse),
            }
          },
          _ => Err(GetDefaultEgoError::InvalidResponse),
        }
      },
      _ => Err(GetDefaultEgoError::InvalidResponse),
    }
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
    cfg: Option<&Configuration>,
    name: &str) -> Result<Ego, ConnectGetDefaultEgoError> {
  let mut is = ttry!(IdentityService::connect(cfg));
  let ret = ttry!(is.get_default_ego(name));
  Ok(ret)
}

