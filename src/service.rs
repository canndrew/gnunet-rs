use std::io::{Reader, Stream, IoResult, IoError, RefReader};
use std::io::net::pipe::UnixStream;
use std::io::util::LimitReader;

use Configuration;

pub struct Service {
  connection: Box<Stream + 'static>,
}

pub enum ServiceConnectError {
  FailedToLoadConfig,
  NotConfigured,
  ConnectionError(IoError),
}

pub enum ProcessMessageResult {
  Continue,
  Reconnect,
  Shutdown,
}

fn hack<'a, T>(x: &'a mut T) -> RefReader<'a, T> where T: Reader {
  x.by_ref()
}

impl Service {
  // TODO: figure out how to make the LimitReader run-time generic in it's type
  pub fn connect<'a, T>(name: &str, cb: T) -> Result<Service, ServiceConnectError> 
      where T: FnMut(u16, LimitReader<&'a mut (Reader + 'static)>) -> ProcessMessageResult,
            T: Send
  {
    let cfg = match Configuration::default() {
      Some(cfg) => cfg,
      None      => return Err(FailedToLoadConfig),
    };
    let unixpath = match cfg.get_value_filename(name, "UNIXPATH") {
      Some(p)   => p,
      None      => return Err(NotConfigured),
    };
    let stream = match UnixStream::connect(&unixpath) {
      Ok(us)  => us,
      Err(e)  => return Err(ConnectionError(e)),
    };
    let mut reader = stream.clone();
    //spawn(move |:| {
    spawn(proc() {
      //TODO: implement reconnection (currently fails)
      loop {
        let len = reader.read_be_u16().unwrap(); // here
        let tpe = reader.read_be_u16().unwrap(); // here
        // TODO: remove referencing `reader.by_ref(()`
        //let lr = LimitReader::new((&mut reader as &Reader).by_ref(), len - 4);
        //let lr = LimitReader::new(Reader::by_ref(&mut reader), (len - 4) as uint);
        let lr = LimitReader::new(&mut reader as &mut Reader, (len - 4) as uint);
        //let lr = LimitReader::new(reader.by_ref(), len as uint - 4);
        match cb(tpe, lr) {
          Continue  => (),
          Reconnect => assert!(false, "Not implemented"),
          Shutdown  => break,
        };
      }
    });
    let ret = Service {
      connection: box stream,
    };
    Ok(ret)
  }
}

impl Writer for Service {
  fn write(&mut self, buf: &[u8]) -> IoResult<()> {
    self.connection.write(buf)
  }
}

/*
impl Drop for Service {
  fn drop(&mut self) {
    self.connection.close_read();
  }
}
*/

