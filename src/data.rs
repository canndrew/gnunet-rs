use std::slice::bytes::MutableByteVector;

static ENCODE_CHARS: [char; 32] = ['0', '1', '2', '3', '4', '5', '6', '7',
                                   '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 
                                   'g', 'h', 'j', 'k', 'm', 'n', 'p', 'q',
                                   'r', 's', 't', 'v', 'w', 'x', 'y', 'z'];

pub fn crockford_encode(buf: &[u8]) -> String {
  let enc_len = (buf.len() * 8 + 4) / 5;
  let mut ret = String::with_capacity(enc_len);

  let mut shift: i32 = 3;
  let mut next_char: u8 = 0;
  for b in buf.iter() {
    while shift >= 0 {
      next_char |= (*b >> shift) & 0x1f;
      let c = ENCODE_CHARS[next_char as usize];
      ret.push(c);
      next_char = 0;
      shift -= 5;
    };
    next_char |= (*b << (-shift)) & 0x1f;
    shift += 8;
  }
  ret
}

pub enum CrockfordDecodeError {
  SizeMismatch {
    encoded_size: usize,
    target_size: usize,
  },
  InvalidChar(char),
}

pub fn crockford_decode(enc: &str, dec: &mut [u8]) -> Result<(), CrockfordDecodeError> {
  let enc_bits = enc.len() * 5;
  let dec_bits = dec.len() * 8;
  
  if enc_bits != dec_bits {
    return Err(CrockfordDecodeError::SizeMismatch {
      encoded_size: enc_bits,
      target_size:  dec_bits,
    });
  };

  dec.set_memory(0u8);

  let mut shift: i32 = 3;
  let mut dp: usize = 0;
  for c in enc.chars() {
    let d = match c {
      '0' | 'O' | 'o' => 0,
      '1' | 'I' | 'i' | 'L' | 'l' => 1,
      '2' => 2,
      '3' => 3,
      '4' => 4,
      '5' => 5,
      '6' => 6,
      '7' => 7,
      '8' => 8,
      '9' => 9,
      'a' | 'A' => 10,
      'b' | 'B' => 11,
      'c' | 'C' => 12,
      'd' | 'D' => 13,
      'e' | 'E' => 14,
      'f' | 'F' => 15,
      'g' | 'G' => 16,
      'h' | 'H' => 17,
      'j' | 'J' => 18,
      'k' | 'K' => 19,
      'm' | 'M' => 20,
      'n' | 'N' => 21,
      'p' | 'P' => 22,
      'q' | 'Q' => 23,
      'r' | 'R' => 24,
      's' | 'S' => 25,
      't' | 'T' => 26,
      'u' | 'U' | 'v' | 'V' => 27,
      'w' | 'W' => 28,
      'x' | 'X' => 29,
      'y' | 'Y' => 30,
      'z' | 'Z' => 31,
      c => return Err(CrockfordDecodeError::InvalidChar(c)),
    };
    if shift < 0 {
      dec[dp] |= d >> (-shift);
      dp += 1;
      shift += 8;
    };
    dec[dp] |= d << shift;
    shift -= 5;
  }
  Ok(())
}

