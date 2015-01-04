extern crate gnunet;

use std::os;
use std::sync::Arc;

use gnunet::{gns, Configuration};

fn main() {
  let args = os::args();
  if args.len() != 2 {
    println!("Usage: example-gns-lookup domain.name.gnu");
    return;
  };
  let config = match Configuration::default() {
    Some(c) => c,
    None    => {
      println!("Error: Config file not found.");
      return;
    },
  };
  match gns::lookup_in_master(Arc::new(config), args[1].as_slice(), gns::RecordType::A, None) {
    Ok(r)   => println!("\t{}", r),
    Err(e)  => println!("Error: {}", e),
  };
}

