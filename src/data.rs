use std::fmt::{self, Write};

static ENCODE_CHARS: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
    'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z',
];

/// Used to wrap a byte slice which can then be crockford base32 encoded using `std::fmt::Display`.
pub struct CrockfordEncode<'a>(pub &'a [u8]);

impl<'a> fmt::Display for CrockfordEncode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CrockfordEncode(buf) => crockford_encode_fmt(f, buf),
        }
    }
}

/// Encodes a byte slice to printable ascii using crockford base32 encoding and returns the result as a
/// `String`.
pub fn crockford_encode(buf: &[u8]) -> String {
    let enc_len = (buf.len() * 8 + 4) / 5;
    let mut ret = String::with_capacity(enc_len);
    write!(ret, "{}", CrockfordEncode(buf)).unwrap();
    ret
}

/// Encodes a byte slice to printable ascii using crockford base32 encoding and writes the encoded
/// data to a `std::fmt::Formatter`.
pub fn crockford_encode_fmt(f: &mut fmt::Formatter, buf: &[u8]) -> fmt::Result {
    let mut shift: i32 = 3;
    let mut next_char: u8 = 0;
    for b in buf.iter() {
        while shift >= 0 {
            next_char |= (*b >> shift) & 0x1f;
            let c = ENCODE_CHARS[next_char as usize];
            try!(fmt::Display::fmt(&c, f));
            next_char = 0;
            shift -= 5;
        }
        next_char |= (*b << (-shift)) & 0x1f;
        shift += 8;
    }
    if shift > 3 {
        let c = ENCODE_CHARS[next_char as usize];
        try!(fmt::Display::fmt(&c, f));
    }

    Ok(())
}

quick_error! {
    /// Errors that occur trying to decode Crockford base32 encoded data.
    #[derive(Debug)]
    pub enum CrockfordDecodeError {
        SizeMismatch {
            encoded_size: usize,
            target_size: usize,
        } {
            display("The size of the encoded data did not match the size of the target buffer. There are {} chars of encoded data but the target buffer is {} bytes long.", encoded_size, target_size)
        }

        InvalidChar { ch: char } {
            display("There was an invalid character in the encoded data. '{}' is not a valid Crockford base32 encoded character. See http://www.crockford.com/wrmg/base32.htm for more info.", ch)
        }

        TrailingBits {
            display("There were trailing 1 bits in the encoded data past the logical end of the data")
        }
    }
}

/// Decodes crockford base32 encoded data and writes the result to a mutable byte slice.
pub fn crockford_decode(enc: &str, dec: &mut [u8]) -> Result<(), CrockfordDecodeError> {
    let enc_len = enc.len();
    let dec_len = dec.len();

    if (enc_len * 5) / 8 != dec_len {
        return Err(CrockfordDecodeError::SizeMismatch {
            encoded_size: enc_len,
            target_size: dec_len,
        });
    };

    for b in dec.iter_mut() {
        *b = 0u8;
    }

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
            c => return Err(CrockfordDecodeError::InvalidChar { ch: c }),
        };
        if shift < 0 {
            dec[dp] |= d >> (-shift);
            dp += 1;
            shift += 8;
            if dp == dec_len {
                return match d << shift {
                    0u8 => Ok(()),
                    _ => Err(CrockfordDecodeError::TrailingBits),
                };
            }
        };
        dec[dp] |= d << shift;
        shift -= 5;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use data::*;

    fn decode_encode(s0: &str, buf: &mut [u8]) {
        println!("decoding: {}", s0);
        crockford_decode(s0, buf).unwrap();
        let s1 = crockford_encode(&buf[..]);
        assert!(s0 == &s1[..], "s0 == \"{}\", s1 == \"{}\"", s0, s1);
    }

    #[test]
    fn tests() {
        let mut buf = [0u8; 6];
        decode_encode("ABCDEFG", &mut buf[..4]);
        decode_encode("ABCDEFGH", &mut buf[..5]);
        decode_encode("ABCDEFGHJ4", &mut buf[..6]);
    }
}
