use std::io::IoResult;
use std::str::FromStr;
use std::mem;
use std::fmt::{self, Show, Formatter};
use std::mem::{uninitialized, size_of, size_of_val};
use std::str::from_utf8;
use std::slice::from_raw_buf;
use libc::{c_void, size_t, c_char};

use ll;
use HashCode;

/// A 256bit ECDSA public key.
#[derive(Copy)]
pub struct EcdsaPublicKey {
  data: ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey,
}

impl EcdsaPublicKey {
  /// Serialize key to a byte stream.
  pub fn serialize<T>(&self, w: &mut T) -> IoResult<()> where T: Writer {
    w.write(&self.data.q_y)
  }

  /// Compute the hash of this key.
  pub fn hash(&self) -> HashCode {
    unsafe {
      HashCode::hash(from_raw_buf(
          &(&self.data as *const ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey as *const u8),
          size_of::<ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey>()
      ))
    }
  }
}

impl FromStr for EcdsaPublicKey {
  fn from_str(s: &str) -> Option<EcdsaPublicKey> {
    let bytes = s.as_bytes();
    unsafe {
      let mut ret: EcdsaPublicKey = mem::uninitialized();
      let res = ll::GNUNET_CRYPTO_ecdsa_public_key_from_string(
          bytes.as_ptr() as *const i8,
          bytes.len() as u64,
          &mut ret.data);
      match res {
        ll::GNUNET_OK => Some(ret),
        _             => None,
      }
    }
  }
}

impl Show for EcdsaPublicKey {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    unsafe {
      const LEN: usize = 52us;
      println!("sizeof == {}", size_of_val(&self.data.q_y));
      assert!(LEN == (size_of_val(&self.data.q_y) * 8 + 4) / 5);
      let mut enc: [u8; LEN] = uninitialized();
      let res = ll::GNUNET_STRINGS_data_to_string(self.data.q_y.as_ptr() as *const c_void,
                                                  self.data.q_y.len() as size_t,
                                                  enc.as_mut_ptr() as *mut c_char,
                                                  52);
      assert!(!res.is_null());
      fmt::String::fmt(from_utf8(&enc).unwrap(), f)
    }
  }
}

impl fmt::String for EcdsaPublicKey {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Show::fmt(self, f)
  }
}

/// A 256bit ECDSA private key.
#[derive(Copy)]
pub struct EcdsaPrivateKey {
  data: ll::Struct_GNUNET_CRYPTO_EcdsaPrivateKey,
}

impl EcdsaPrivateKey {
  /// Serialize this key to a byte stream.
  pub fn serialize<T>(&self, w: &mut T) -> IoResult<()> where T: Writer {
    w.write(&self.data.d)
  }

  /// Deserialize a from a byte stream.
  pub fn deserialize<T>(r: &mut T) -> IoResult<EcdsaPrivateKey> where T: Reader {
    let mut ret: EcdsaPrivateKey = unsafe { uninitialized() };
    try!(r.read(&mut ret.data.d));
    Ok(ret)
  }

  /// Get the corresponding public key to this private key.
  pub fn get_public(&self) -> EcdsaPublicKey {
    unsafe {
      let mut ret: ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey = uninitialized();
      ll::GNUNET_CRYPTO_ecdsa_key_get_public(&self.data, &mut ret);
      EcdsaPublicKey {
        data: ret,
      }
    }
  }

  /// Return the private key of the global, anonymous user.
  pub fn anonymous() -> EcdsaPrivateKey {
    //let anon = ll::GNUNET_CRYPTO_ecdsa_key_get_anonymous();
    unsafe {
      EcdsaPrivateKey {
        data: *ll::GNUNET_CRYPTO_ecdsa_key_get_anonymous(),
      }
    }
  }
}

impl Clone for EcdsaPrivateKey {
  fn clone(&self) -> EcdsaPrivateKey {
    EcdsaPrivateKey {
      data: ll::Struct_GNUNET_CRYPTO_EcdsaPrivateKey {
        d: self.data.d,
      },
    }
  }
}

/*
impl FromStr for EcdsaPrivateKey {
  fn from_str(s: &str) -> Option<EcdsaPrivateKey> {
    let bytes = s.as_bytes();
    unsafe {
      let mut ret: EcdsaPrivateKey = mem::uninitialized();
      let res = ll::GNUNET_CRYPTO_ecdsa_private_key_from_string(
          bytes.as_ptr() as *const i8,
          bytes.len() as u64,
          &mut ret.data);
      match res {
        ll::GNUNET_OK => Some(ret),
        _             => None,
      }
    }
  }
}
*/

#[test]
fn test_ecdsa_to_from_string() {
  use EcdsaPublicKey;

  //let s0: &str = "JK55QA8JLAL64MBO8UM209KE93M9JBBO7M2UB8M3M03FKRFSUOMG";
  let s0: &str = "JK55QA8J1A164MB08VM209KE93M9JBB07M2VB8M3M03FKRFSV0MG";
  let key: Option<EcdsaPublicKey> = FromStr::from_str(s0);
  let s1: String = format!("{}", key.unwrap());
  println!("{} {}", s0, s0.len());
  println!("{} {}", s1, s1.len());
  assert!(s0 == &s1[]);
}

