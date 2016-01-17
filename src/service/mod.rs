//! Module for communicating with GNUnet services. Implements the parts of the GNUnet IPC protocols
//! that are common to all services.

use std::io::{self, Write, Cursor};
use std::thread;
use std::net::Shutdown;
use unix_socket::UnixStream;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use configuration::{self, Cfg};
use util::io::ReadUtil;

/*
pub struct Service<'c> {
  //connection: Box<Stream + 'static>,
  //pub connection: Box<UnixStream>,
  pub connection: UnixStream,
  pub cfg: &'c Cfg,
}
*/

/// Created by `service::connect`. Used to read messages from a GNUnet service.
pub struct ServiceReader {
    /// The underlying socket wrapped by `ServiceReader`. This is a read-only socket.
    pub connection: UnixStream, // TODO: should be UnixReader
}

/// Created by `service::connect`. Used to send messages to a GNUnet service.
pub struct ServiceWriter {
    /// The underlying socket wrapped by `ServiceWriter`. This is a write-only socket.
    pub connection: UnixStream, // TODO: should be UnixWriter
}

/// Callbacks passed to `ServiceReader::spawn_callback_loop` return a `ProcessMessageResult` to
/// tell the callback loop what action to take next.
#[derive(Copy, Clone)]
pub enum ProcessMessageResult {
  /// Continue talking to the service and passing received messages to the callback.
  Continue,
  /// Attempt to reconnect to the service.
  Reconnect,
  /// Exit the callback loop, shutting down it's thread.
  Shutdown,
}

/// Error that can be generated when attempting to connect to a service.
error_def! ConnectError {
    NotConfigured { #[from] cause: configuration::CfgGetFilenameError }
        => "The configuration does not describe how to connect to the service"
            ("Config does not contain an entry for UNIXPATH in the service's section: {}", cause),
    Io { #[from] cause: io::Error }
        => "There was an I/O error communicating with the service" ("Specifically {}", cause),
}

/// Attempt to connect to the local GNUnet service named `name`.
///
/// eg. `connect(cfg, "arm")` will attempt to connect to the locally-running `gnunet-arm` service
/// using the congfiguration details (eg. socket address, port etc.) in `cfg`.
pub fn connect(cfg: &Cfg, name: &str) -> Result<(ServiceReader, ServiceWriter), ConnectError> {
  let unixpath = try!(cfg.get_filename(name, "UNIXPATH"));

  // TODO: use UnixStream::split() instead when it exists
  let path = unixpath.into_os_string().into_string().unwrap();
  let in_stream = try!(UnixStream::connect(path));
  let out_stream = try!(in_stream.try_clone());

  let r = ServiceReader {
    connection: in_stream,
  };
  let w = ServiceWriter {
    connection: out_stream,
  };
  Ok((r, w))
}

/// Error that can be generated when attempting to receive data from a service.
error_def! ReadMessageError {
  Io { #[from] cause: io::Error } => "There was an I/O error communicating with the service" ("Specifically {}", cause),
  ShortMessage { len: u16 }       => "The message received from the service was too short" ("Length was {} bytes.", len),
  Disconnected                    => "The service disconnected unexpectedly",
}
byteorder_error_chain! {ReadMessageError}

impl ServiceReader {
  pub fn spawn_callback_loop<F>(mut self, mut cb: F) -> Result<ServiceReadLoop, io::Error>
      where F: FnMut(u16, Cursor<Vec<u8>>) -> ProcessMessageResult,
            F: Send,
            F: 'static
  {
    let reader = try!(self.connection.try_clone());
    let callback_loop = thread::spawn(move || -> ServiceReader {
      //TODO: implement reconnection (currently fails)
      loop {
        let (tpe, mr) = match self.read_message() {
          Ok(x)   => x,
          Err(_)  => return self, // TODO: reconnect
        };
        match cb(tpe, mr) {
          ProcessMessageResult::Continue  => (),
          ProcessMessageResult::Reconnect => return self, //TODO: auto reconnect
          ProcessMessageResult::Shutdown  => return self,
        };
      }
    });
    Ok(ServiceReadLoop {
      reader:        reader,
      _callback_loop: callback_loop,
    })
  }

  pub fn read_message(&mut self) -> Result<(u16, Cursor<Vec<u8>>), ReadMessageError> {
    let len = try!(self.connection.read_u16::<BigEndian>());
    if len < 4 {
      return Err(ReadMessageError::ShortMessage { len: len });
    };
    let v = try!(self.connection.read_exact_alloc(len as usize - 2));
    let mut mr = Cursor::new(v);
    let tpe = try!(mr.read_u16::<BigEndian>());
    Ok((tpe, mr))
  }
}

impl ServiceWriter {
  pub fn write_message<'a>(&'a mut self, len: u16, tpe: u16) -> MessageWriter<'a> {
    assert!(len >= 4);
    let v = Vec::with_capacity(len as usize);
    let mut mw = Cursor::new(v);
    mw.write_u16::<BigEndian>(len).unwrap();
    mw.write_u16::<BigEndian>(tpe).unwrap();
    MessageWriter {
      service_writer: self,
      mw: mw,
    }
  }
}

/// Used to form messsages before sending them to the GNUnet service.
pub struct MessageWriter<'a> {
  service_writer: &'a mut ServiceWriter,
  mw: Cursor<Vec<u8>>,
}

impl<'a> MessageWriter<'a> {
  /// Finish the message and transmit it to the service.
  pub fn send(self) -> Result<(), io::Error> {
    let v = self.mw.into_inner();
    assert!(v.len() == v.capacity());
    self.service_writer.connection.write_all(&v[..])
  }
}

impl<'a> Write for MessageWriter<'a> {
  fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
    self.mw.write(buf)
  }

  fn flush(&mut self) -> Result<(), io::Error> {
    Ok(())
  }
}

/// A thread that loops, recieving messages from the service and passing them to a callback.
/// Created with `ServiceReader::spawn_callback_loop`.
pub struct ServiceReadLoop {
  reader: UnixStream,
  _callback_loop: thread::JoinHandle<ServiceReader>,
}

impl ServiceReadLoop {
  /*
  fn join(mut self) -> ServiceReader {
    let _ = self.reader.shutdown(Shutdown::Read);
    self.callback_loop.join().unwrap()
  }
  */
}

impl Drop for ServiceReadLoop {
  fn drop(&mut self) {
    let _ = self.reader.shutdown(Shutdown::Read);
    //let _ = self.callback_loop.join();
  }
}

/*
// TODO: why do I need this unsafe bizo?
#[unsafe_destructor]
impl Drop for ServiceReader {
  fn drop(&mut self) {
    // cause the loop task to exit
    let _ = self.connection.close_read();
  }
}
*/

