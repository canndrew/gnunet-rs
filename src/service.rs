use std::io::{Reader, IoResult, IoError, EndOfFile};
use std::io::net::pipe::UnixStream;
use std::io::util::LimitReader;

use FromError;

use Configuration;

pub struct Service {
  //connection: Box<Stream + 'static>,
  pub connection: Box<UnixStream>,
  pub cfg: Configuration,
}

#[deriving(Show)]
pub enum ServiceConnectError {
  FailedToLoadConfig,
  NotConfigured,
  ConnectionError(IoError),
  InvalidResponse,
}
error_chain!(IoError, ServiceConnectError, ConnectionError)

pub enum ProcessMessageResult {
  ServiceContinue,
  ServiceReconnect,
  ServiceShutdown,
}

impl Service {
  pub fn connect(cfg: Option<Configuration>, name: &str) -> Result<Service, ServiceConnectError> {
    let cfg = match cfg {
      Some(cfg) => cfg,
      None      => match Configuration::default() {
        Some(cfg) => cfg,
        None      => return Err(FailedToLoadConfig),
      },
    };
    let unixpath = match cfg.get_value_filename(name, "UNIXPATH") {
      Some(p)   => p,
      None      => return Err(NotConfigured),
    };
    let stream = ttry!(UnixStream::connect(&unixpath));
    Ok(Service {
      connection: box stream,
      cfg: cfg,
    })
  }

  // TODO: figure out how to make the LimitReader run-time generic in it's type
  /*
  pub fn connect<'a, T>(name: &str, cb: T) -> Result<Service, ServiceConnectError> 
      where T: FnMut(u16, LimitReader<&'a mut (Reader + 'static)>) -> ProcessMessageResult,
            T: Send
  {
  */
  pub fn connect_loop<T>(cfg: Option<Configuration>, name: &str, mut cb: T) -> Result<Service, ServiceConnectError> 
      where T: FnMut(u16, LimitReader<UnixStream>) -> ProcessMessageResult,
            T: Send
  {
    let service = ttry!(Service::connect(cfg, name));
    let mut reader = (*service.connection).clone();
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
          ServiceContinue  => /* TODO: need lifetimes on closures to do this: assert!(lr.limit() == 0, "callback did not read entire message") */ (),
          ServiceReconnect => return, //TODO: auto reconnect
          ServiceShutdown  => return,
        };
      }
    });
    Ok(service)
  }
}

impl Writer for Service {
  fn write(&mut self, buf: &[u8]) -> IoResult<()> {
    self.connection.write(buf)
  }
}

impl Reader for Service {
  fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
    self.connection.read(buf)
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

