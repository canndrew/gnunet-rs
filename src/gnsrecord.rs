use std::io::IoResult;
use std::from_str::FromStr;
use std::fmt::{Show, Formatter};
use std::fmt;
use std::c_str::CString;
use libc::c_void;

use ll;
use FromError;

pub enum GNSRecordType {
  A       = 1,
  NS      = 2,
  CNAME   = 5,
  SOA     = 6,
  PTR     = 12,
  MX      = 15,
  TXT     = 16,
  AAAA    = 28,
  TLSA    = 52,

  PKEY    = 65536,
  NICK    = 65537,
  LEHO    = 65538,
  VPN     = 65539,
  GNS2DNS = 65540,
}

impl GNSRecordType {
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

#[allow(dead_code)]
pub struct GNSRecord {
  data: ll::Struct_GNUNET_GNSRECORD_Data,
  buff: Vec<u8>,
}

impl GNSRecord {
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

