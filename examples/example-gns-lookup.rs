extern crate gnunet;

use std::env;

use gnunet::{gns, Configuration};

fn print_help(executable: String) {
  println!("Usage: {} domain.name.gnu", executable);
}

fn main() {
  let mut args = env::args();
  let executable = args.next().unwrap();
  let domain     = match args.next() {
    Some(domain)  => domain,
    None          => {
      println!("Missing domain name");
      print_help(executable);
      return;
    },
  };
  match args.next() {
    Some(x) => {
      println!("Unexpected argument: {}", x);
      print_help(executable);
      return;
    },
    None  => (),
  }
  let config = match Configuration::default() {
    Some(c) => c,
    None    => {
      println!("Error: Config file not found.");
      return;
    },
  };
  match gns::lookup_in_master(&config, &domain[..], gns::RecordType::A, None) {
    Ok(r)   => println!("\t{}", r),
    Err(e)  => println!("Error: {}", e),
  };
}

