extern crate gnunet;

fn main() {
  let config = match gnunet::Configuration::default() {
    Some(c) => c,
    None    => {
      println!("Error: Config file not found.");
      return;
    },
  };
  let peers = match gnunet::iterate_peers(&config) {
    Ok(peers) => peers,
    Err(e)    => {
      println!("Failed to iterate peers: {}", e);
      return;
    },
  };
  for result in peers {
    match result {
      Err(e)  => {
        println!("Error receiving peer info: {}", e);
        return;
      },
      Ok((peerinfo, hello)) => {
        println!("Peer: {}", peerinfo);
        if let Some(hello) = hello {
          println!("Hello: {}", hello);
        };
        println!("");
      },
    }
  }
}

