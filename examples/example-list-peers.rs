extern crate gnunet;

fn main() {
    let config = gnunet::Cfg::default().unwrap();
    let peers = gnunet::iterate_peers(&config).unwrap();
    for result in peers {
        let (peerinfo, hello) = result.unwrap();
        println!("Peer: {}", peerinfo);
        if let Some(hello) = hello {
            println!("Hello: {}", hello);
        };
        println!("");
    };

    let local_id = gnunet::self_id(&config).unwrap();
    println!("Our id is: {}", local_id);
}

