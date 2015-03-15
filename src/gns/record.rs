use std::old_io::IoResult;
use std::str::FromStr;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::ffi::c_str_to_bytes;
use std::str::from_utf8;
//use std::c_str::CString;
use libc::{free, c_char, c_void};

use ll;
use self::RecordType::*;
use gns::error::*;

/// An enum of the different GNS record types.
///
/// Some of these records exist in the legacy DNS (but are still used in GNS). Others are specific
/// to GNS. These are marked **Legacy** and **GNS** respectively.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecordType {
  /// **Legacy.** Address record. Stores a 32bit IPv4 address.
  A       = 1,
  /// **Legacy.** Name server record. Delegates a DNS zone to use the given authoritative name servers.
  NS      = 2,
  /// **Legacy.** Canonical name record. Alias of one name to another.
  CNAME   = 5,
  /// **Legacy.** Start of authority record. Specifies authoritative information about a DNS zone.
  SOA     = 6,
  /// **Legacy.** Pointer record. Pointer to a canonical name.
  PTR     = 12,
  /// **Legacy.** Mail exchange record. Maps a domain name to a list of message transfer agents for that
  /// domain.
  MX      = 15,
  /// **Legacy.** Text record. Used to store human-readable data and various forms of machine-readable data.
  TXT     = 16,
  /// **Legacy.** Address record. Stores a 128bit IPv6 address.
  AAAA    = 28,
  /// **Legacy.** TLSA certificate association. A record for DNS-based Authentication of Named Entities (DANE).
  TLSA    = 52,

  /// **GNS.** Petname key record. Used to delegate to other users' zones and give those zones a petname.
  PKEY    = 65536,
  /// **GNS.** Nickname record. Used to give a zone a name.
  NICK    = 65537,
  /// **GNS.** Legacy hostname record.
  LEHO    = 65538,
  /// **GNS.** Virtual public network record.
  VPN     = 65539,
  /// **GNS.** GNS2DNS record. Used to delegate authority to a legacy DNS zone.
  GNS2DNS = 65540,
}

impl RecordType {
  /// Creates a RecordType from it's record type number.
  ///
  /// # Example
  ///
  /// ```rust
  /// use gnunet::gns::RecordType::{mod, A};
  ///
  /// let x = RecordType::from_u32(1);
  /// let y = RecordType::from_u32(1234);
  /// assert!(x == Some(A));
  /// assert!(y == None);
  /// ```
  pub fn from_u32(x: u32) -> Option<RecordType> {
    Some(match x {
      1 => A,
      2 => NS,
      5 => CNAME,
      6 => SOA,
      12 => PTR,
      15 => MX,
      16 => TXT,
      28 => AAAA,
      52 => TLSA,

      65536 => PKEY,
      65537 => NICK,
      65538 => LEHO,
      65539 => VPN,
      65540 => GNS2DNS,

      _ => return None,
    })
  }
}

impl FromStr for RecordType {
  type Err = RecordTypeFromStrError;

  fn from_str(s: &str) -> Result<RecordType, RecordTypeFromStrError> {
    match s {
      "A"       => Ok(A),
      "NS"      => Ok(NS),
      "CNAME"   => Ok(CNAME),
      "SOA"     => Ok(SOA),
      "PTR"     => Ok(PTR),
      "MX"      => Ok(MX),
      "TXT"     => Ok(TXT),
      "AAAA"    => Ok(AAAA),
      "TLSA"    => Ok(TLSA),

      "PKEY"    => Ok(PKEY),
      "NICK"    => Ok(NICK),
      "LEHO"    => Ok(LEHO),
      "VPN"     => Ok(VPN),
      "GNS2DNS" => Ok(GNS2DNS),
      _         => Err(RecordTypeFromStrError),
    }
  }
}

impl fmt::Display for RecordType {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Debug::fmt(self, f)
  }
}

/// A record in the GNU Name System.
#[allow(dead_code)]
pub struct Record {
  data: ll::Struct_GNUNET_GNSRECORD_Data,
  buff: Vec<u8>,
}

impl Record {
  /// Deserialize a record from a byte stream.
  pub fn deserialize<T>(reader: &mut T) -> IoResult<Record> where T: Reader {
    let expiration_time = try!(reader.read_be_u64());
    let data_size = try!(reader.read_be_u32()) as u64;
    let record_type = try!(reader.read_be_u32());
    let flags = try!(reader.read_be_u32());
    let buff = try!(reader.read_exact(data_size as usize));
    let data = buff.as_ptr() as *const c_void;

    Ok(Record {
      data: ll::Struct_GNUNET_GNSRECORD_Data {
        data:             data,
        expiration_time:  expiration_time,
        data_size:        data_size,
        record_type:      record_type,
        flags:            flags,
      },
      buff: buff,
    })
  }

  /// Get the type of a record.
  pub fn record_type(&self) -> RecordType {
    RecordType::from_u32(self.data.record_type).unwrap()
  }
}

impl Debug for Record {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let tpe = self.data.record_type;
    try!(write!(f, "{:?}: ", RecordType::from_u32(tpe).unwrap()));
    unsafe {
      let cs = ll::GNUNET_GNSRECORD_value_to_string(tpe, self.data.data, self.data.data_size);
      match cs.is_null() {
        true  => write!(f, "<malformed record data>"),
        false => {
          let constified = cs as *const c_char;
          let s = from_utf8(c_str_to_bytes(&constified));
          let ret = match s {
            Ok(ss)  => write!(f, "{}", ss),
            Err(_)  => write!(f, "<invalid utf8>"),
          };
          // TODO: use the c-string wrapper that automatically dealloces when it exists
          free(cs as *mut c_void);
          ret
        },
      }
    }
  }
}

impl fmt::Display for Record {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Debug::fmt(self, f)
  }
}

