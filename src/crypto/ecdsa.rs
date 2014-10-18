use std::io::IoResult;
use std::from_str::FromStr;
use std::mem;
use std::fmt::{Show, Formatter};
use std::c_str::CString;
use std::fmt;

use ll;

pub struct EcdsaPublicKey {
  data: ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey,
}

impl EcdsaPublicKey {
  pub fn serialize<T>(&self, w: &mut T) -> IoResult<()> where T: Writer {
    w.write(self.data.q_y)
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
      let s = ll::GNUNET_CRYPTO_ecdsa_public_key_to_string(&self.data);
      //CString::new(s, true).as_str().unwrap().fmt(f)
      CString::new(s as *const i8, true).as_str().unwrap().fmt(f)
    }
  }
}

pub struct EcdsaPrivateKey {
  data: ll::Struct_GNUNET_CRYPTO_EcdsaPrivateKey,
}

impl EcdsaPrivateKey {
  pub fn serialize<T>(&self, w: &mut T) -> IoResult<()> where T: Writer {
    w.write(self.data.d)
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

  let s0: &str = "JK55QA8JLAL64MBO8UM209KE93M9JBBO7M2UB8M3M03FKRFSUOMG";
  let key: Option<EcdsaPublicKey> = FromStr::from_str(s0);
  let s: String = format!("{}", key.unwrap());
  let s1: &str = s.as_slice();
  println!("{} {}", s0, s0.len());
  println!("{} {}", s1, s1.len());
  assert!(s0.equiv(&s1));
}

