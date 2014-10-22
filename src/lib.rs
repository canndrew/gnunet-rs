#![feature(while_let)]
#![feature(macro_rules)]
#![feature(if_let)]
#![feature(overloaded_calls)]
#![feature(unboxed_closures)]
#![feature(tuple_indexing)]
#![feature(slicing_syntax)]
#![feature(unsafe_destructor)]
#![feature(default_type_params)]

extern crate libc;
extern crate sync;

pub use configuration::Configuration;
//pub use crypto::{EcdsaPublicKey, EcdsaPrivateKey};
pub use crypto::ecdsa::{EcdsaPublicKey, EcdsaPrivateKey};
pub use crypto::hashcode::HashCode;

pub use service::ServiceConnectError;
pub use gnsrecord::{GNSRecord, GNSRecordType};
pub use gns::{GNS, LocalOptions};
pub use identity::{Ego, IdentityService};

macro_rules! ttry (
    ($expr:expr) => ({
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(FromError::from_error(err))
        }
    })
)

macro_rules! error_chain (
  ($from:ty, $to:ty, $f:expr) => (
    impl FromError<$from> for $to {
      fn from_error(e: $from) -> $to {
        $f(e)
      }
    }
  )
)

#[allow(dead_code, non_camel_case_types, non_snake_case, non_uppercase_statics)]
mod ll;

mod service;
mod configuration;
pub mod gns;
pub mod gnsrecord;
mod crypto;
pub mod identity;

pub trait FromError<E> {
  fn from_error(x: E) -> Self;
}

impl<E> FromError<E> for E {
  fn from_error(e: E) -> E {
    e
  }
}

