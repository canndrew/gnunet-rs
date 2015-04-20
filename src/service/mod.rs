use std::io::{self, Write, Cursor};
use std::thread;
use std::net::Shutdown;
use unix_socket::UnixStream;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use Configuration;
use util::io::ReadUtil;

pub use self::error::*;

mod error;

/*
pub struct Service<'c> {
  //connection: Box<Stream + 'static>,
  //pub connection: Box<UnixStream>,
  pub connection: UnixStream,
  pub cfg: &'c Configuration,
}
*/

pub struct ServiceReader {
  pub connection: UnixStream, // TODO: should be UnixReader
}

pub struct ServiceWriter {
  pub connection: UnixStream, // TODO: should be UnixWriter
}

#[derive(Copy, Clone)]
pub enum ProcessMessageResult {
  Continue,
  Reconnect,
  Shutdown,
}

pub fn connect(cfg: &Configuration, name: &str) -> Result<(ServiceReader, ServiceWriter), ConnectError> {
  let unixpath = match cfg.get_value_filename(name, "UNIXPATH") {
    Some(p)   => p,
    None      => return Err(ConnectError::NotConfigured),
  };

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

impl ServiceReader {
  pub fn spawn_callback_loop<F>(mut self, mut cb: F) -> Result<ServiceReadLoop, io::Error>
      where F: FnMut(u16, Cursor<Vec<u8>>) -> ProcessMessageResult,
            F: Send,
            F: 'static
  {
    let reader = try!(self.connection.try_clone());
    let callback_loop = thread::scoped(move || -> ServiceReader {
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
      return Err(ReadMessageError::ShortMessage(len));
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

pub struct MessageWriter<'a> {
  service_writer: &'a mut ServiceWriter,
  mw: Cursor<Vec<u8>>,
}

impl<'a> MessageWriter<'a> {
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

pub struct ServiceReadLoop {
  reader: UnixStream,
  _callback_loop: thread::JoinGuard<'static, ServiceReader>,
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

