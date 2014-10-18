#![feature(while_let)]
#![feature(macro_rules)]
#![feature(if_let)]
#![feature(overloaded_calls)]
#![feature(unboxed_closures)]
#![feature(tuple_indexing)]

extern crate libc;

pub use configuration::Configuration;
pub use crypto::{EcdsaPublicKey, EcdsaPrivateKey};

//pub use service::ServiceConnectError;
//pub use gnsrecord::{GNSRecord, GNSRecordType};
//pub use gns::GNS;

macro_rules! ttry (
    ($expr:expr) => ({
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(FromError::from_error(err))
        }
    })
)

#[allow(dead_code, non_camel_case_types, non_snake_case, non_uppercase_statics)]
mod ll;

//mod service;
mod configuration;
//mod gns;
//mod gnsrecord;
mod crypto;

pub trait FromError<E> {
  fn from_error(x: E) -> Self;
}

impl<E> FromError<E> for E {
  fn from_error(e: E) -> E {
    e
  }
}

