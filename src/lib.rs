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
//! use gnunet::{Configuration, gns};
//!
//! let c = Configuration::default().unwrap();
//! let r = gns::lookup_in_master(&c, "www.gnu", gns::RecordType::A, None).unwrap();
//! println!("Got the following IPv4 record for www.gnu: {}", r);
//! ```

#![feature(unboxed_closures)]
#![feature(std_misc)]
#![feature(core)]
#![feature(libc)]
#![feature(hash)]
#![feature(scoped)]
#![feature(plugin)]

#![plugin(error_def)]

#![allow(deprecated)]

#![crate_name = "gnunet"]

extern crate libc;
extern crate unix_socket;
extern crate rand;
extern crate byteorder;
extern crate crypto as rcrypto;
extern crate num;

pub use configuration::Configuration;
pub use crypto::{EcdsaPublicKey, EcdsaPrivateKey, HashCode};

pub use gns::{Record, RecordType};
pub use gns::{GNS, LocalOptions};
pub use identity::{Ego, IdentityService};
pub use hello::Hello;
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
  ($t:ident) => (
    impl From<::byteorder::Error> for $t {
      #[inline]
      fn from(e: ::byteorder::Error) -> $t {
        match e {
          ::byteorder::Error::UnexpectedEOF => $t::Disconnected,
          ::byteorder::Error::Io(e)         => $t::Io { cause: e },
        }
      }
    }
  )
}

//const HOMEPAGE: &'static str = "http://github.com/canndrew/gnunet-rs";

#[allow(dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals, raw_pointer_derive)]
mod ll;

pub mod service;
mod configuration;
pub mod gns;
//pub mod dht;
mod crypto;
pub mod identity;
mod util;
pub mod peerinfo;
pub mod hello;
//pub mod cadet;
pub mod data;
pub mod transport;

