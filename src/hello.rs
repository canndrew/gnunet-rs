use byteorder::{BigEndian, ReadBytesExt};
use std::fmt;
use std::io::{self, Read};

use PeerIdentity;

#[derive(Debug)]
pub struct Hello {
    /// Use this peer in F2F mode. Do not gossip this hello.
    pub friend_only: bool,

    /// The identity of the peer.
    pub id: PeerIdentity,
}

quick_error! {
    #[derive(Debug)]
    pub enum HelloDeserializeError {
        ShortMessage {
            display("Unexpected EOF when deserializing the hello")
        }

        Io { cause: io::Error } {
            display("There was an I/O error reading the hello. Error: {}", cause)
            cause(cause)
            from(cause: io::Error) -> { cause: cause }
        }
    }
}

impl Hello {
    pub fn deserialize<R>(r: &mut R) -> Result<Hello, HelloDeserializeError>
    where
        R: Read,
    {
        let friend_only = match r.read_u32::<BigEndian>() {
            Ok(x) => x != 0,
            Err(e) => {
                return Err(match e.kind() {
                    ::std::io::ErrorKind::UnexpectedEof => HelloDeserializeError::ShortMessage,
                    _ => HelloDeserializeError::Io { cause: e },
                })
            }
        };
        let id = try!(PeerIdentity::deserialize(r));
        Ok(Hello {
            friend_only: friend_only,
            id: id,
        })
    }
}

impl fmt::Display for Hello {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hello!")
    }
}
