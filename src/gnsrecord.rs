use std::io::IoResult;
use std::mem::uninitialized;
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

pub struct GNSRecord {
  data: ll::Struct_GNUNET_GNSRECORD_Data,
  buff: Vec<u8>,
}

impl GNSRecord {
  pub fn deserialize<T>(reader: &mut T) -> IoResult<GNSRecord> where T: Reader {
    let buff: Vec<u8> = Vec::new();

    let expiration_time = ttry!(reader.read_be_u64());
    let data_size = ttry!(reader.read_be_u32()) as u64;
    buff.reserve_exact(data_size as uint);
    let record_type = ttry!(reader.read_be_u32());
    let flags = ttry!(reader.read_be_u32());
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
}

