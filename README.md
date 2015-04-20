gnunet-rs
=========

GNUnet bindings for Rust.

*Note:* This library is for interacting with a locally running GNUnet peer. It
does not implement a peer itself. It is also FAR from complete. Only a few
rudimentry features are implemented. You cannot, for example, use this for
peer-to-peer communication (yet).

Features implemented so far:

  * Parsing GNUnet config files.
  * Retrieving peer info from the peerinfo service.
  * Performing GNS lookups.
  * Performing identity ego lookups.

Next on the list:

  * DHT bindings.
  * Cadet (peer-to-peer) bindings.
  * Datastore bindings.

See http://canndrew.org/rust-doc/gnunet for documentation.
See examples directory for example code.
Feedback and pull requests are encouraged!

