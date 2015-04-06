use std::old_io::{Reader, Writer, IoResult, EndOfFile};
use std::old_io::net::pipe::UnixStream;
use std::old_io::util::LimitReader;
use std::old_io::{MemReader, MemWriter};
use std::thread;
use std::result::Result;

use Configuration;

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
  let out_stream = in_stream.clone();

  let r = ServiceReader {
    connection: in_stream,
  };
  let w = ServiceWriter {
    connection: out_stream,
  };
  Ok((r, w))
}

impl ServiceReader {
  pub fn spawn_callback_loop<F>(mut self, mut cb: F) -> ServiceReadLoop
      where F: FnMut(u16, LimitReader<UnixStream>) -> ProcessMessageResult,
            F: Send,
            F: 'static
  {
    let reader = self.connection.clone();
    let callback_loop = thread::scoped(move || -> ServiceReader {
      //TODO: implement reconnection (currently fails)
      loop {
        let len = match self.connection.read_be_u16() {
          Ok(x)   => x,
          Err(e)  => match e.kind {
            EndOfFile => return self,
            _         => return self, //TODO: auto reconnect
          },
        };
        // TODO: remove these unwraps, do auto reconnect of failure
        let tpe = self.connection.read_be_u16().unwrap();
        let lr = LimitReader::new(self.connection.clone(), len as usize); // TODO: get rid of this clone
        match cb(tpe, lr) {
          ProcessMessageResult::Continue  => /* TODO: need lifetimes on closures to do this: assert!(lr.limit() == 0, "callback did not read entire message") */ (),
          ProcessMessageResult::Reconnect => return self, //TODO: auto reconnect
          ProcessMessageResult::Shutdown  => return self,
        };
      }
    });
    ServiceReadLoop {
      reader: reader,
      _callback_loop: callback_loop,
    }
  }

  pub fn read_message(&mut self) -> Result<(u16, MemReader), ReadMessageError> {
    let len = try!(self.connection.read_be_u16());
    if len < 4 {
      return Err(ReadMessageError::ShortMessage(len));
    }
    let v = try!(self.connection.read_exact(len as usize - 2));
    let mut mr = MemReader::new(v);
    let tpe = try!(mr.read_be_u16());
    Ok((tpe, mr))
  }
}

impl ServiceWriter {
  pub fn write_message<'a>(&'a mut self, len: u16, tpe: u16) -> MessageWriter<'a> {
    assert!(len >= 4);
    let mut mw = MemWriter::with_capacity(len as usize);
    mw.write_be_u16(len).unwrap();
    mw.write_be_u16(tpe).unwrap();
    MessageWriter {
      service_writer: self,
      mw: mw,
    }
  }
}

pub struct MessageWriter<'a> {
  service_writer: &'a mut ServiceWriter,
  mw: MemWriter,
}

impl<'a> MessageWriter<'a> {
  pub fn send(self) -> IoResult<()> {
    let v = self.mw.into_inner();
    assert!(v.len() == v.capacity());
    self.service_writer.connection.write(&v[..])
  }
}

impl<'a> Writer for MessageWriter<'a> {
  fn write(&mut self, buf: &[u8]) -> IoResult<()> {
    self.mw.write(buf)
  }

  fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
    self.mw.write_all(buf)
  }
}

pub struct ServiceReadLoop {
  reader: UnixStream,
  _callback_loop: thread::JoinGuard<'static, ServiceReader>,
}

impl ServiceReadLoop {
  /*
  fn join(mut self) -> ServiceReader {
    let _ = self.reader.close_read();
    self.callback_loop.join().ok().unwrap()
  }
  */
}

impl Drop for ServiceReadLoop {
  fn drop(&mut self) {
    let _ = self.reader.close_read();
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

