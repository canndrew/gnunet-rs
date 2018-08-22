//! # Rust interface for GNUnet. GNU's framework for secure peer-to-peer networking.
//!
//! This library does not implement a GNUnet peer. You must have the GNUnet software installed and
//! running in order to use this library.
//!
//! # Examples
//!
//! Perform a [GNS](https://gnunet.org/gns) lookup.
//!
//! ```rust
//! use gnunet::{Cfg, gns};
//!
//! let c = Cfg::default().unwrap();
//! let r = gns::lookup_in_master(&c, "www.gnu", gns::RecordType::A, None).unwrap();
//! println!("Got the following IPv4 record for www.gnu: {}", r);
//! ```

//#![feature(unboxed_closures)]
//#![feature(libc)]
//#![feature(plugin)]
//#![feature(into_cow)]
#![crate_name = "gnunet"]

extern crate byteorder;
extern crate crypto as rcrypto;
extern crate libc;
extern crate num;
extern crate rand;
extern crate regex;
extern crate unix_socket;

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate lazy_static;

pub use configuration::Cfg;
pub use crypto::{EcdsaPrivateKey, EcdsaPublicKey, HashCode};

pub use gns::{LocalOptions, GNS};
pub use gns::{Record, RecordType};
pub use hello::Hello;
pub use identity::{Ego, IdentityService};
pub use peerinfo::{iterate_peers, self_id, PeerIdentity};
//pub use dht::DHT;

/*
macro_rules! error_chain {
  ($from:ty, $to:ident, $f:ident) => (
    impl From<$from> for $to {
      fn from(e: $from) -> $to {
        $to::$f(e)
      }
    }
  )
}

macro_rules! byteorder_error_chain {
  ($t:ident) => (
    impl From<byteorder::Error> for $t {
      #[inline]
      fn from(e: byteorder::Error) -> $t {
        match e {
          byteorder::Error::UnexpectedEOF => $t::Disconnected,
          byteorder::Error::Io(e)         => $t::Io(e),
        }
      }
    }
  )
}
*/

macro_rules! byteorder_error_chain {
    ($t:ident) => {
        impl From<::std::io::Error> for $t {
            #[inline]
            fn from(e: ::std::io::Error) -> $t {
                match e.kind() {
                    ::std::io::ErrorKind::UnexpectedEof => $t::Disconnected,
                    _ => $t::Io { cause: e },
                }
            }
        }
    };
}

/*
macro_rules! unwrap_result {
    ($e:expr) => {
        match $e {
            Ok(o) => o,
            Err(ref e) => {
                ::print_error(e, file!(), line!());
                panic!();
            }
        }
    };
}
*/

#[cfg(test)]
fn print_error<E: ::std::error::Error>(error: &E, file: &str, line: u32) {
    println!("{}:{}: unwrap_result! called on an Err", file, line);
    let mut err: Option<&::std::error::Error> = Some(error);
    while let Some(e) = err {
        println!("    {}", e);
        err = e.cause();
    }
}

//const HOMEPAGE: &'static str = "http://github.com/canndrew/gnunet-rs";

#[allow(dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ll;

pub mod configuration;
pub mod gns;
pub mod paths;
pub mod service;
pub mod time;
//pub mod dht;
mod crypto;
pub mod hello;
pub mod identity;
pub mod peerinfo;
mod util;
//pub mod cadet;
pub mod data;
pub mod transport;
