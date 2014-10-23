use std::io::IoResult;
use std::from_str::FromStr;
use std::fmt::{Show, Formatter};
use std::fmt;
use std::c_str::CString;
use libc::c_void;

use ll;
use FromError;

/// An enum of the different GNS record types.
///
/// Some of these records exist in the legacy DNS (but are still used in GNS). Others are specific
/// to GNS. These are marked **Legacy** and **GNS** respectively.
#[deriving(PartialEq)]
pub enum GNSRecordType {
  /// **Legacy.** Address record. Stores a 32bit IPv4 address.
  A       = 1,
  /// **Legacy**. Name server record. Delegates a DNS zone to use the given authoritative name servers.
  NS      = 2,
  /// **Legacy**. Canonical name record. Alias of one name to another.
  CNAME   = 5,
  /// **Legacy**. Start of authority record. Specifies authoritative information about a DNS zone.
  SOA     = 6,
  /// **Legacy**. Pointer record. Pointer to a canonical name.
  PTR     = 12,
  /// **Legacy**. Mail exchange record. Maps a domain name to a list of message transfer agents for that
  /// domain.
  MX      = 15,
  /// **Legacy**. Text record. Used to store human-readable data and various forms of machine-readable data.
  TXT     = 16,
  /// **Legacy**. Address record. Stores a 128bit IPv6 address.
  AAAA    = 28,
  /// **Legacy**. TLSA certificate association. A record for DNS-based Authentication of Named Entities (DANE).
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

impl GNSRecordType {
  /// Creates a GNSRecordType from it's record type number.
  ///
  /// # Example
  ///
  /// ```rust
  /// use gnunet::gnsrecord::{GNSRecordType, A};
  ///
  /// let x = GNSRecordType::from_u32(1);
  /// let y = GNSRecordType::from_u32(1234);
  /// assert!(x == Some(A));
  /// assert!(y == None);
  /// ```
  pub fn from_u32(x: u32) -> Option<GNSRecordType> {
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

impl FromStr for GNSRecordType {
  fn from_str(s: &str) -> Option<GNSRecordType> {
    match s {
      "A"       => Some(A),
      "NS"      => Some(NS),
      "CNAME"   => Some(CNAME),
      "SOA"     => Some(SOA),
      "PTR"     => Some(PTR),
      "MX"      => Some(MX),
      "TXT"     => Some(TXT),
      "AAAA"    => Some(AAAA),
      "TLSA"    => Some(TLSA),

      "PKEY"    => Some(PKEY),
      "NICK"    => Some(NICK),
      "LEHO"    => Some(LEHO),
      "VPN"     => Some(VPN),
      "GNS2DNS" => Some(GNS2DNS),
      _         => None,
    }
  }
}

impl Show for GNSRecordType {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      &A       => "A",
      &NS      => "NS",
      &CNAME   => "CNAME",
      &SOA     => "SOA",
      &PTR     => "PTR",
      &MX      => "MX",
      &TXT     => "TXT",
      &AAAA    => "AAAA",
      &TLSA    => "TLSA",

      &PKEY    => "PKEY",
      &NICK    => "NICK",
      &LEHO    => "LEHO",
      &VPN     => "VPN",
      &GNS2DNS => "GNS2DNS",
    }.fmt(f)
  }
}

/// A record in the GNU Name System.
#[allow(dead_code)]
pub struct GNSRecord {
  data: ll::Struct_GNUNET_GNSRECORD_Data,
  buff: Vec<u8>,
}

impl GNSRecord {
  /// Deserialize a record from a byte stream.
  pub fn deserialize<T>(reader: &mut T) -> IoResult<GNSRecord> where T: Reader {
    let expiration_time = ttry!(reader.read_be_u64());
    let data_size = ttry!(reader.read_be_u32()) as u64;
    let record_type = ttry!(reader.read_be_u32());
    let flags = ttry!(reader.read_be_u32());
    let buff = ttry!(reader.read_exact(data_size as uint));
    let data = buff.as_ptr() as *const c_void;

    Ok(GNSRecord {
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
  pub fn record_type(&self) -> GNSRecordType {
    GNSRecordType::from_u32(self.data.record_type).unwrap()
  }
}

impl Show for GNSRecord {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let tpe = self.data.record_type;
    try!(write!(f, "{}: ", GNSRecordType::from_u32(tpe).unwrap()));
    unsafe {
      let cs = ll::GNUNET_GNSRECORD_value_to_string(tpe, self.data.data, self.data.data_size);
      match cs.is_null() {
        true  => write!(f, "<malformed record data>"),
        false => {
          let cs = CString::new_owned(cs);
          write!(f, "{}", cs)
        },
      }
    }
  }
}

