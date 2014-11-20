use std::io::{Reader, IoResult, EndOfFile};
use std::io::net::pipe::UnixStream;
use std::io::util::LimitReader;
use std::io::{MemReader, MemWriter};

use Configuration;

pub use self::error::*;

mod error;

pub struct Service {
  //connection: Box<Stream + 'static>,
  pub connection: Box<UnixStream>,
  pub cfg: Configuration,
}

pub enum ProcessMessageResult {
  Continue,
  Reconnect,
  Shutdown,
}

impl Service {
  pub fn connect(cfg: Option<&Configuration>, name: &str) -> Result<Service, ConnectError> {
    let cfg = match cfg {
      Some(cfg) => cfg.clone(),
      None      => match Configuration::default() {
        Some(cfg) => cfg,
        None      => return Err(ConnectError::FailedToLoadConfig),
      },
    };
    let unixpath = match cfg.get_value_filename(name, "UNIXPATH") {
      Some(p)   => p,
      None      => return Err(ConnectError::NotConfigured),
    };
    let stream = ttry!(UnixStream::connect(&unixpath));
    Ok(Service {
      connection: box stream,
      cfg: cfg,
    })
  }

  pub fn init_callback_loop<T>(&mut self, mut cb: T)
      where T: FnMut(u16, LimitReader<UnixStream>) -> ProcessMessageResult,
            T: Send
  {
    let mut reader = (*self.connection).clone();
    //spawn(move |:| {
    spawn(proc() {
      //TODO: implement reconnection (currently fails)
      loop {
        let len = match reader.read_be_u16() {
          Ok(x)   => x,
          Err(e)  => match e.kind {
            EndOfFile => return,
            _         => return, //TODO: auto reconnect
          },
        };
        let tpe = reader.read_be_u16().unwrap(); // here
        // TODO: remove referencing `reader.by_ref(()`
        //let lr = LimitReader::new(&mut reader as &mut Reader, (len - 4) as uint);
        let lr = LimitReader::new(reader.clone(), (len - 4) as uint);
        match cb(tpe, lr) {
          ProcessMessageResult::Continue  => /* TODO: need lifetimes on closures to do this: assert!(lr.limit() == 0, "callback did not read entire message") */ (),
          ProcessMessageResult::Reconnect => return, //TODO: auto reconnect
          ProcessMessageResult::Shutdown  => return,
        };
      }
    });
  }

  pub fn read_message(&mut self) -> Result<(u16, MemReader), ReadMessageError> {
    let len = ttry!(self.connection.read_be_u16());
    if len < 4 {
      return Err(ReadMessageError::ShortMessage(len));
    }
    let v = ttry!(self.connection.read_exact(len as uint - 2));
    let mut mr = MemReader::new(v);
    let tpe = ttry!(mr.read_be_u16());
    Ok((tpe, mr))
  }

  pub fn write_message(&mut self, len: u16, tpe: u16) -> MessageWriter {
    assert!(len >= 4);
    let mut mw = MemWriter::with_capacity(len as uint);
    mw.write_be_u16(len).unwrap();
    mw.write_be_u16(tpe).unwrap();
    MessageWriter {
      service: self,
      mw: mw,
    }
  }
}

pub struct MessageWriter<'a> {
  service: &'a mut Service,
  mw: MemWriter,
}

impl<'a> MessageWriter<'a> {
  pub fn send(self) -> IoResult<()> {
    let v = self.mw.unwrap();
    assert!(v.len() == v.capacity());
    self.service.connection.write(v[])
  }
}

impl<'a> Writer for MessageWriter<'a> {
  fn write(&mut self, buf: &[u8]) -> IoResult<()> {
    self.mw.write(buf)
  }
}

// TODO: why do I need this unsafe bizo?
#[unsafe_destructor]
impl Drop for Service {
  fn drop(&mut self) {
    // cause the loop task to exit
    let _ = self.connection.close_read();
  }
}

