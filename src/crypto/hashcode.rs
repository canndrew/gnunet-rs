pub struct HashCode {
  data: ll::Struct_GNUNET_HashCode;
}

impl HashCode {
  fn hash(buf: &[u8]) -> HashCode {
  }

  fn distance(&self, other: &HashCode) -> u32 {
  }

  fn bit(&self, idx: uint) -> bool {
  }

  fn matching_prefix_len(&self, other: &self) -> uint {
  }

  fn xor_cmp(&self, h1: &HashCode, h2: &HashCode) -> Ordering {
  }
}

impl FromStr for HashCode {
  fn from_str(s: &str) -> Option<HashCode> {
  }
}

impl Rand for HashCode {
  fn rand<R: Rng>(rng: &mut R) -> HashCode {
  }
}

impl Add<HashCode> for HashCode {
  fn add(&self, rhs: &HashCode) -> HashCode) {
  }
}

impl Sub<HashCode> for HashCode {
  fn sub(&self, rhs: &HashCode) -> HashCode {
  }
}

impl BitXor<HashCode> for HashCode {
  fn bitxor(&self, rhs: &HashCode) -> HashCode {
  }
}

impl PartialOrd for HashCode {
  fn partial_cmp(&self, other: &HashCode) -> Option<Ordering> {
  }
}

impl Ord for HashCode {
  fn cmp(&self, other: &HashCode) -> Ordering {
  }
}

impl Iterator<bool> for HashCode {
}

