use libc::{c_char, c_void, size_t};
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read, Write};
use std::mem;
use std::mem::{size_of, size_of_val, uninitialized};
use std::slice::from_raw_parts;
use std::str::from_utf8;
use std::str::FromStr;

use crypto::hashcode::HashCode;
use ll;

/// A 256bit ECDSA public key.
#[derive(Copy, Clone)]
pub struct EcdsaPublicKey {
    data: ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey,
}

impl EcdsaPublicKey {
    /// Serialize key to a byte stream.
    pub fn serialize<T>(&self, w: &mut T) -> Result<(), io::Error>
    where
        T: Write,
    {
        w.write_all(&self.data.q_y)
    }

    /// Compute the hash of this key.
    pub fn hash(&self) -> HashCode {
        unsafe {
            HashCode::from_buffer(from_raw_parts(
                &self.data as *const ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey as *const u8,
                size_of::<ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey>(),
            ))
        }
    }
}

/// Error generated when attempting to parse an ecdsa public key
quick_error! {
  #[derive(Debug)]
  pub enum EcdsaPublicKeyFromStrError {
      ParsingFailed {
          display("Failed to parse the string as an ecdsa public key")
      }
  }
}

impl FromStr for EcdsaPublicKey {
    type Err = EcdsaPublicKeyFromStrError;

    fn from_str(s: &str) -> Result<EcdsaPublicKey, EcdsaPublicKeyFromStrError> {
        let bytes = s.as_bytes();
        unsafe {
            let mut ret: EcdsaPublicKey = mem::uninitialized();
            let res = ll::GNUNET_CRYPTO_ecdsa_public_key_from_string(
                bytes.as_ptr() as *const i8,
                bytes.len() as usize,
                &mut ret.data,
            );
            match res {
                ll::GNUNET_OK => Ok(ret),
                _ => Err(EcdsaPublicKeyFromStrError::ParsingFailed),
            }
        }
    }
}

impl Debug for EcdsaPublicKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        unsafe {
            const LEN: usize = 52usize;
            assert!(LEN == (size_of_val(&self.data.q_y) * 8 + 4) / 5);
            let mut enc: [u8; LEN] = uninitialized();
            let res = ll::GNUNET_STRINGS_data_to_string(
                self.data.q_y.as_ptr() as *const c_void,
                self.data.q_y.len() as size_t,
                enc.as_mut_ptr() as *mut c_char,
                enc.len() as size_t,
            );
            assert!(!res.is_null());
            fmt::Display::fmt(from_utf8(&enc).unwrap(), f)
        }
    }
}

impl fmt::Display for EcdsaPublicKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

/// A 256bit ECDSA private key.
#[derive(Copy)]
pub struct EcdsaPrivateKey {
    data: ll::Struct_GNUNET_CRYPTO_EcdsaPrivateKey,
}

impl EcdsaPrivateKey {
    /// Serialize this key to a byte stream.
    pub fn serialize<T>(&self, w: &mut T) -> Result<(), io::Error>
    where
        T: Write,
    {
        w.write_all(&self.data.d)
    }

    /// Deserialize a from a byte stream.
    pub fn deserialize<T>(r: &mut T) -> Result<EcdsaPrivateKey, io::Error>
    where
        T: Read,
    {
        let mut ret: EcdsaPrivateKey = unsafe { uninitialized() };
        try!(r.read_exact(&mut ret.data.d[..]));
        Ok(ret)
    }

    /// Get the corresponding public key to this private key.
    pub fn get_public(&self) -> EcdsaPublicKey {
        unsafe {
            let mut ret: ll::Struct_GNUNET_CRYPTO_EcdsaPublicKey = uninitialized();
            ll::GNUNET_CRYPTO_ecdsa_key_get_public(&self.data, &mut ret);
            EcdsaPublicKey { data: ret }
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
            data: ll::Struct_GNUNET_CRYPTO_EcdsaPrivateKey { d: self.data.d },
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
    let key: EcdsaPublicKey = FromStr::from_str(s0).unwrap();
    let s1: String = format!("{}", key);
    println!("{} {}", s0, s0.len());
    println!("{} {}", s1, s1.len());
    assert!(s0 == &s1[..]);
}
