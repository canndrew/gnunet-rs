extern crate gnunet;

use std::os;

use gnunet::gnsrecord;
use gnunet::gns;

fn main() {
  let args = os::args();
  if args.len() != 2 {
    println!("Usage: example-gns-lookup domain.name.gnu");
    return;
  }
  let rx = gns::lookup(None, args[1].as_slice(), gnsrecord::A, None).unwrap();
  println!("\t{}", rx);
}

