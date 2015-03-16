use std::old_io::{Reader, BytesReader};
use std::str::from_utf8;
use std::collections::HashMap;
use std::num::ToPrimitive;

use ll;
use EcdsaPrivateKey;
use EcdsaPublicKey;
use HashCode;
use service::{self, ServiceReader, ServiceWriter};
use Configuration;
use util::CStringReader;
pub use self::error::*;
use util::ReadCStringError;

mod error;

/// A GNUnet identity.
///
/// An ego consists of a public/private key pair and a name.
#[derive(Clone)]
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
  service_reader: ServiceReader,
  service_writer: ServiceWriter,
  egos: HashMap<HashCode, Ego>,
}

impl IdentityService {
  /// Connect to the identity service.
  ///
  /// Returns either a handle to the identity service or a `ServiceConnectError`. `cfg` contains
  /// the configuration to use to connect to the service.
  pub fn connect(cfg: &Configuration) -> Result<IdentityService, ConnectError> {
    /*
    let (get_tx, get_rx) = channel::<(String, Sender<Option<Ego>>>();
    let service = try!(Service::connect("identity", move |&mut: tpe: u16, mut reader: LimitReader<UnixStream>| -> ProcessMessageResult {
      loop {
        
      }
    }));
    */
    let (mut service_reader, mut service_writer) = try!(service::connect(cfg, "identity"));
    let mut egos: HashMap<HashCode, Ego> = HashMap::new();
    {
      let mw = service_writer.write_message(4, ll::GNUNET_MESSAGE_TYPE_IDENTITY_START);
      try!(mw.send());
    };
    loop {
      let (tpe, mut mr) = try!(service_reader.read_message());
      match tpe {
        ll::GNUNET_MESSAGE_TYPE_IDENTITY_UPDATE => {
          let name_len = try!(mr.read_be_u16());
          let eol = try!(mr.read_be_u16());
          if eol != 0 {
            break;
          };
          let pk = try!(EcdsaPrivateKey::deserialize(&mut mr));
          let mut v: Vec<u8> = Vec::with_capacity(name_len as usize);
          for r in mr.bytes() {
            let b = try!(r);
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
      service_reader: service_reader,
      service_writer: service_writer,
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
  /// use gnunet::{Configuration, IdentityService};
  ///
  /// let config = Configuration::default().unwrap();
  /// let mut ids = IdentityService::connect(&config).unwrap();
  /// let ego = ids.get_default_ego("gns-master").unwrap();
  /// ```
  pub fn get_default_ego(&mut self, name: &str) -> Result<Ego, GetDefaultEgoError> {
    let name_len = name.len();

    let msg_length = match (8 + name_len + 1).to_u16() {
      Some(l) => l,
      None    => return Err(GetDefaultEgoError::NameTooLong(name.to_string())),
    };
    {
      let mut mw = self.service_writer.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_IDENTITY_GET_DEFAULT);
      try!(mw.write_be_u16((name_len + 1) as u16));
      try!(mw.write_be_u16(0));
      try!(mw.write(name.as_bytes()));
      try!(mw.write_u8(0u8));
      try!(mw.send());
    };

    let (tpe, mut mr) = try!(self.service_reader.read_message());
    match tpe {
      ll::GNUNET_MESSAGE_TYPE_IDENTITY_RESULT_CODE => {
        try!(mr.read_be_u32());
        match mr.read_cstring() {
          Err(e)  => match e {
            ReadCStringError::Io(e)       => Err(GetDefaultEgoError::Io(e)),
            ReadCStringError::FromUtf8(e) => Err(GetDefaultEgoError::MalformedErrorResponse(e)),
          },
          Ok(s) => Err(GetDefaultEgoError::ServiceResponse(s)),
        }
      },
      ll::GNUNET_MESSAGE_TYPE_IDENTITY_SET_DEFAULT => match try!(mr.read_be_u16()) {
        0 => Err(GetDefaultEgoError::InvalidResponse),
        reply_name_len => {
          let zero = try!(mr.read_be_u16());
          match zero {
            0 => {
              let pk = try!(EcdsaPrivateKey::deserialize(&mut mr));
              let s = try!(mr.read_cstring_with_len((reply_name_len - 1) as usize));
              match &s[..] == name {
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
/// use gnunet::{Configuration, identity};
///
/// let config = Configuration::default().unwrap();
/// let ego = identity::get_default_ego(&config, "gns-master").unwrap();
/// ```
///
/// # Note
///
/// This a convenience function that connects to the identity service, does the query, then
/// disconnects. If you want to do multiple queries you should connect to the service with
/// `IdentityService::connect` then use that handle to do the queries.
pub fn get_default_ego(
    cfg: &Configuration,
    name: &str) -> Result<Ego, ConnectGetDefaultEgoError> {
  let mut is = try!(IdentityService::connect(cfg));
  let ret = try!(is.get_default_ego(name));
  Ok(ret)
}

