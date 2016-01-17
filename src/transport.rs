use std::io::{self, Write};
use byteorder::{WriteBytesExt, BigEndian};

use service::{self, ReadMessageError};
use hello::HelloDeserializeError;
use Hello;
use Cfg;
use ll;

pub struct TransportService {
  //service_reader: ServiceReader,
  //service_writer: ServiceWriter,
  our_hello:      Hello,
}

error_def! TransportServiceInitError {
  NonHelloMessage { ty: u16 }
    => "Expected a HELLO message from the service but received a different message type" ("Received message type {} instead.", ty),
  Io { #[from] cause: io::Error }
    => "There was an I/O error communicating with the service" ("Error: {}", cause),
  ReadMessage { #[from] cause: ReadMessageError }
    => "Failed to receive a message from the service" ("Reason: {}", cause),
  Connect { #[from] cause: service::ConnectError } 
    => "Failed to connect to the transport service" ("Reason: {}", cause),
  HelloDeserialize { #[from] cause: HelloDeserializeError }
    => "Failed to serialize the hello message from the service" ("Reason {}", cause),
}

impl TransportService {
  pub fn init(cfg: &Cfg) -> Result<TransportService, TransportServiceInitError> {
    let (mut sr, mut sw) = try!(service::connect(cfg, "transport"));
    let msg_length = 2 + 4 + 32;
    {
      let mut mw = sw.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_TRANSPORT_START);
      mw.write_u32::<BigEndian>(0).unwrap();
      let null_peer_id = [0; 32];
      mw.write(&null_peer_id[..]).unwrap();
      try!(mw.send());
    };
    let (ty, mut mr) = try!(sr.read_message());
    if ty != ll::GNUNET_MESSAGE_TYPE_HELLO {
      return Err(TransportServiceInitError::NonHelloMessage { ty: ty });
    };
    let hello = try!(Hello::deserialize(&mut mr));
    Ok(TransportService {
      //service_reader: sr,
      //service_writer: sw,
      our_hello:      hello,
    })
  }
}

pub fn self_hello(cfg: &Cfg) -> Result<Hello, TransportServiceInitError> {
  let ts = try!(TransportService::init(cfg));
  Ok(ts.our_hello)
}

