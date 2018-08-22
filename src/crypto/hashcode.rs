use rcrypto::digest::Digest;
use rcrypto::sha2::Sha512;
use std::cmp::Ordering;
use std::fmt;
use std::hash;
use std::mem;
use std::num::Wrapping;
use std::ops::{Add, BitXor, Sub};
use std::slice;
use std::str::FromStr;

use data;

/// A 512-bit hashcode used in various places throughout GNUnet.
#[derive(PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct HashCode {
    data: [u32; 16],
}

impl HashCode {
    /// Get the data underlying buffer as a buffer
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const u8, 64) }
    }

    /// Get the data underlying buffer as a mutable buffer
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut u8, 64) }
    }

    /// Create a HashCode by computing the sha512 hash of a buffer.
    pub fn from_buffer(buf: &[u8]) -> HashCode {
        let mut ret = HashCode {
            data: unsafe { mem::uninitialized() },
        };
        let mut hasher = Sha512::new();
        hasher.input(buf);
        hasher.result(ret.as_mut_slice());
        ret
    }

    /// Compute the distance between two hashes.
    pub fn distance(&self, other: &HashCode) -> u32 {
        let a1 = Wrapping(self.data[1]);
        let b1 = Wrapping(other.data[1]);
        let x1 = (a1 - b1) >> 16;
        let x2 = (b1 - a1) >> 16;
        (x1 * x2).0
    }

    /// Get the nth bit of a HashCode.
    ///
    /// # Panics
    ///
    /// Panics if `idx >= 512`.
    pub fn get_bit(&self, idx: u32) -> bool {
        assert!(idx < 512);
        let idx = idx as usize;
        (self.as_slice()[idx >> 3] & (1 << (idx & 7))) > 0
    }

    /// Compute the length (in bits) of the common prefix of two hashes. ie. two identical hashes
    /// will return a value of 512u32 while two hashes that vary in the first bit will return a value
    /// of 0u32.
    pub fn matching_prefix_len(&self, other: &HashCode) -> u32 {
        for i in 0..512 {
            if self.get_bit(i) != other.get_bit(i) {
                return i;
            };
        }
        512
    }

    /// Determine which hash is closer to `self` in the XOR metric (Kademlia). Returns `Less` if
    /// `h1` is smaller than `h2` relative to `self`. ie. if `(h1 ^ self) < (h2 ^ self)`. Otherwise
    /// returns `Greater` or `Equal` if `h1` is greater than or equal to `h2` relative to `self`.
    pub fn xor_cmp(&self, h0: &HashCode, h1: &HashCode) -> Ordering {
        use std::cmp::Ordering::*;

        let mut i = 16;
        while i > 0 {
            i -= 1;
            let s = self.data[i];
            let x0 = h0.data[i];
            let x1 = h1.data[i];
            let d0 = x0 ^ s;
            let d1 = x1 ^ s;
            match d0.cmp(&d1) {
                Less => return Less,
                Greater => return Greater,
                _ => (),
            }
        }
        Equal
    }
}

impl fmt::Display for HashCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        data::crockford_encode_fmt(f, self.as_slice())
    }
}

impl fmt::Debug for HashCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for HashCode {
    type Err = data::CrockfordDecodeError;

    fn from_str(s: &str) -> Result<HashCode, data::CrockfordDecodeError> {
        let mut ret = HashCode {
            data: unsafe { mem::uninitialized() },
        };
        try!(data::crockford_decode(s, ret.as_mut_slice()));
        Ok(ret)
    }
}

/*
impl rand::Rand for HashCode {
    fn rand<R>(rng: &mut R) -> HashCode
    where
        R: rand::Rng,
    {
        let mut ret = HashCode {
            data: unsafe { mem::uninitialized() },
        };
        for u in ret.data.iter_mut() {
            *u = rng.next_u32();
        }
        ret
    }
}
*/

impl Add<HashCode> for HashCode {
    type Output = HashCode;

    fn add(self, rhs: HashCode) -> HashCode {
        let mut ret = HashCode {
            data: unsafe { mem::uninitialized() },
        };
        for i in 0..ret.data.len() {
            ret.data[i] = (Wrapping(self.data[i]) + Wrapping(rhs.data[i])).0;
        }
        ret
    }
}

impl Sub<HashCode> for HashCode {
    type Output = HashCode;

    fn sub(self, rhs: HashCode) -> HashCode {
        let mut ret = HashCode {
            data: unsafe { mem::uninitialized() },
        };
        for i in 0..ret.data.len() {
            ret.data[i] = (Wrapping(self.data[i]) - Wrapping(rhs.data[i])).0;
        }
        ret
    }
}

impl BitXor<HashCode> for HashCode {
    type Output = HashCode;

    fn bitxor(self, rhs: HashCode) -> HashCode {
        let mut ret = HashCode {
            data: unsafe { mem::uninitialized() },
        };
        for i in 0..ret.data.len() {
            ret.data[i] = self.data[i] ^ rhs.data[i];
        }
        ret
    }
}

impl hash::Hash for HashCode {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.data.hash(state)
    }

    fn hash_slice<H>(data: &[HashCode], state: &mut H)
    where
        H: hash::Hasher,
    {
        for h in data.iter() {
            h.hash(state);
        }
    }
}

#[test]
fn test_hashcode_to_from_string() {
    let s0: &str = "RMKN0V1JNA3PVC1148D6J10STVG94A8A651N0K849CF1RT6BGF26AMMT14GMDMNRDFSJRJME61KJ31DFBV12R1TPQJE64155132QN5G";
    let hc: HashCode = FromStr::from_str(s0).unwrap();
    let s: String = format!("{}", hc);
    let s1: &str = &s[..];
    assert!(s0 == s1, "s0 == {}, s1 == {}", s0, s1);
}

#[test]
fn test_hashcode_rand_add_sub() {
    use rand::weak_rng;

    let mut rng = weak_rng();
    let h0: HashCode = rng.gen();
    let h1: HashCode = rng.gen();
    let diff = h1.clone() - h0.clone();
    let sum = h0.clone() + diff;
    assert!(sum == h1);
}
