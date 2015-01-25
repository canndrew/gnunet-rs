use std::mem::{uninitialized, size_of_val};
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::str::{from_utf8, FromStr};
use std::rand::{Rand, Rng};
use std::hash::Hash;
use std::hash;
use std::cmp::Ordering;
use std::ops::{Add, Sub, BitXor};
use libc::{c_char, c_uint, c_void, size_t};

use ll;

/// A 512bit hash code used in various places throughout GNUnet.
#[derive(Copy)]
pub struct HashCode {
  data: ll::Struct_GNUNET_HashCode,
}

impl HashCode {
  /// Compute the hash of a buffer.
  pub fn hash(buf: &[u8]) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      ll::GNUNET_CRYPTO_hash(buf.as_ptr() as *const c_void, buf.len() as size_t, &mut ret);
      HashCode {
        data: ret,
      }
    }
  }

  /// Compute the distance between two hashes.
  pub fn distance(&self, other: &HashCode) -> u32 {
    unsafe {
      ll::GNUNET_CRYPTO_hash_distance_u32(&self.data, &other.data) as u32
    }
  }

  /// Get the nth bit of a 512bit hash code.
  ///
  /// # Panics
  ///
  /// Panics if `idx >= 512`.
  pub fn get_bit(&self, idx: u32) -> bool {
    assert!(idx < 512);
    unsafe {
      ll::GNUNET_CRYPTO_hash_get_bit(&self.data, idx as c_uint) == 1
    }
  }

  /// Compute the length (in bits) of the common prefix of two hashes. ie. two identical hashes
  /// will return a value of 512u32 while two hashes that vary in the first bit will return a value
  /// of 0u32.
  pub fn matching_prefix_len(&self, other: &HashCode) -> u32 {
    unsafe {
      ll::GNUNET_CRYPTO_hash_matching_bits(&self.data, &other.data) as u32
    }
  }

  /// Determine which hash is closer to `self` in the XOR metric (Kademlia). Returns `Less` if
  /// `h1` is smaller than `h2` relative to `self`. ie. if `(h1 ^ self) < (h2 ^ self)`. Otherwise
  /// returns `Greater` or `Equal` if `h1` is greater than or equal to `h2` relative to `self`.
  pub fn xor_cmp(&self, h1: &HashCode, h2: &HashCode) -> Ordering {
    unsafe {
      match ll::GNUNET_CRYPTO_hash_xorcmp(&h1.data, &h2.data, &self.data) {
        -1  => Ordering::Less,
        0   => Ordering::Equal,
        1   => Ordering::Greater,
        _   => panic!("Invalid value returned by ll::GNUNET_CRYPTO_hash_xorcmp"),
      }
    }
  }
}

impl PartialEq for HashCode {
  fn eq(&self, other: &HashCode) -> bool {
    self.data.bits == other.data.bits
  }
}

impl Eq for HashCode {}

impl Clone for HashCode {
  fn clone(&self) -> HashCode {
    HashCode {
      data: ll::Struct_GNUNET_HashCode {
        bits: self.data.bits,
      },
    }
  }
}

impl Debug for HashCode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    unsafe {
      const LEN: usize = 103us;
      assert!(LEN == (size_of_val(&self.data.bits) * 8 + 4) / 5);
      let mut enc: [u8; LEN] = uninitialized();
      let res = ll::GNUNET_STRINGS_data_to_string(self.data.bits.as_ptr() as *const c_void,
                                                  size_of_val(&self.data.bits) as size_t,
                                                  enc.as_mut_ptr() as *mut c_char,
                                                  enc.len() as size_t);
      assert!(!res.is_null());
      fmt::Display::fmt(from_utf8(&enc).unwrap(), f)
    }
  }
}

impl fmt::Display for HashCode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Debug::fmt(self, f)
  }
}

impl FromStr for HashCode {
  fn from_str(s: &str) -> Option<HashCode> {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      let res = ll::GNUNET_CRYPTO_hash_from_string2(s.as_ptr() as *const i8, s.len() as size_t, &mut ret);
      match res {
        ll::GNUNET_OK => Some(HashCode {
            data: ret,
        }),
        _ => None,
      }
    }
  }
}

impl Rand for HashCode {
  fn rand<R: Rng>(rng: &mut R) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      for u in ret.bits.iter_mut() {
        *u = rng.next_u32();
      };
      HashCode {
        data: ret,
      }
    }
  }
}

impl Add<HashCode> for HashCode {
  type Output = HashCode;

  fn add(self, rhs: HashCode) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      ll::GNUNET_CRYPTO_hash_sum(&self.data, &rhs.data, &mut ret);
      HashCode {
        data: ret,
      }
    }
  }
}

impl Sub<HashCode> for HashCode {
  type Output = HashCode;

  fn sub(self, rhs: HashCode) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      ll::GNUNET_CRYPTO_hash_difference(&rhs.data, &self.data, &mut ret);
      HashCode {
        data: ret,
      }
    }
  }
}

impl BitXor<HashCode> for HashCode {
  type Output = HashCode;

  fn bitxor(self, rhs: HashCode) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      ll::GNUNET_CRYPTO_hash_xor(&self.data, &rhs.data, &mut ret);
      HashCode {
        data: ret,
      }
    }
  }
}

impl PartialOrd for HashCode {
  fn partial_cmp(&self, other: &HashCode) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for HashCode {
  fn cmp(&self, other: &HashCode) -> Ordering {
    unsafe {
      match ll::GNUNET_CRYPTO_hash_cmp(&self.data, &other.data) {
        -1  => Ordering::Less,
        0   => Ordering::Equal,
        1   => Ordering::Greater,
        _   => panic!("Invalid return from GNUNET_CRYPTO_hash_cmp"),
      }
    }
  }
}

impl<S: hash::Writer + hash::Hasher> Hash<S> for HashCode {
  fn hash(&self, state: &mut S) {
    self.data.bits.hash(state)
  }
}

#[test]
fn test_hashcode_to_from_string() {
  let s0: &str = "RMKN0V1JNA3PVC1148D6J10STVG94A8A651N0K849CF1RT6BGF26AMMT14GMDMNRDFSJRJME61KJ31DFBV12R1TPQJE64155132QN5G";
  let hc: Option<HashCode> = FromStr::from_str(s0);
  let s: String = format!("{}", hc.unwrap());
  let s1: &str = s.as_slice();
  println!("s0: {}", s0);
  println!("s1: {}", s1);
  assert!(s0 == s1);
}

#[test]
fn test_hashcode_rand_add_sub() {
  use std::rand::weak_rng;

  let mut rng = weak_rng();
  let h0: HashCode = rng.gen();
  let h1: HashCode = rng.gen();
  let diff = h1 - h0;
  let sum = h0 + diff;
  assert!(sum == h1);
}

