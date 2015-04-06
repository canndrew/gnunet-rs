use std::ptr;
use libc::{c_char, c_void, size_t, free};
use std::mem::uninitialized;
use std::time::Duration;
use std::str::FromStr;
use std::str::from_utf8;
use std::num::ToPrimitive;
use std::ffi::CStr;
use std::path::{PathBuf, AsPath};

use ll;
use util::to_c_path;
use configuration::error::*;

/*
 * TODO: Make this all nicer once Index is reformed
 */

/*
#[deriving(Clone)]
pub enum ConfigValue {
  Int(u64),
  Float(f32),
  Duration(Duration),
  Size(u64),
  String(String),
  Choice(String),
  Filename(Path),
}
*/

/// A set of key-value pairs containing the configuration of a local GNUnet daemon.
///
/// You need one of these objects to connect to any GNUnet service as it contains (among other
/// things) information on how to connect to the service.
pub struct Configuration {
  data: *mut ll::Struct_GNUNET_CONFIGURATION_Handle,
}
unsafe impl Send for Configuration {}
unsafe impl Sync for Configuration {}

/*
pub struct ConfigSection<'s> {
  conf: &mut ll::Struct_GNUNET_CONFIGURATION_Handle,
  name: &'s str,
}
*/

impl Configuration {
  /// Generate an empty configuration
  pub fn empty() -> Configuration {
    unsafe {
      let cfg = ll::GNUNET_CONFIGURATION_create();
      Configuration {
        data: cfg,
      }
    }
  }

  /// Generate a default configuration.
  ///
  /// This will find and load the system-wide GNUnet config file. If it cannot find the file then
  /// `None` is returned.
  pub fn default() -> Option<Configuration> {
    let cfg = Configuration::empty();
    unsafe {
      match ll::GNUNET_CONFIGURATION_load(cfg.data, ptr::null()) {
        ll::GNUNET_OK => Some(cfg),
        _             => None,
      }
    }
  }

  /// Load a configuration file.
  ///
  /// This starts by loading the system-wide config file then loads any additional options in
  /// `filename`. If either the system-wide config or `filename` cannot be found then `None` is
  /// returned.
  pub fn load<P: AsPath + ?Sized>(filename: &P) -> Result<Configuration, ConfigurationLoadError> {
    let cpath = match to_c_path(filename) {
      Ok(cpath) => cpath,
      Err(e)    => return Err(ConfigurationLoadError::BadPath(e)),
    };
    let cfg = Configuration::empty();
    unsafe {
      let r = ll::GNUNET_CONFIGURATION_load(cfg.data, cpath.as_ptr());
      match r {
        ll::GNUNET_OK => Ok(cfg),
        _             => Err(ConfigurationLoadError::NoSuchFile),
      }
    }
  }

  /// Get an int from the config in the form of a `u32`. `None` is returned if the `section` or
  /// `option` are not found or if they don't index an int.
  pub fn get_value_int(&self, section: &str, option: &str) -> Option<u64> {
    unsafe {
      let mut n: u64 = uninitialized();
      let r = ll::GNUNET_CONFIGURATION_get_value_number(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          section.as_ptr() as *const c_char,
          option.as_ptr() as *const c_char,
          &mut n);
      match r {
        ll::GNUNET_OK => Some(n),
        _             => None,
      }
    }
  }

  /// Get a float from the config in the form of a `f32`. `None` is returned if the `section` or
  /// `option` are not found or if they don't index a float.
  pub fn get_value_float(&self, section: &str, option: &str) -> Option<f32> {
    unsafe {
      let mut f: f32 = uninitialized();
      let r = ll::GNUNET_CONFIGURATION_get_value_float(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          section.as_ptr() as *const c_char,
          option.as_ptr() as *const c_char,
          &mut f);
      match r {
        ll::GNUNET_OK => Some(f),
        _             => None,
      }
    }
  }

  /// Get a duration from the config in the form of a `Duration`. `None` is returned if the
  /// `section` or `option` are not found in the config or if they don't index a duration.
  pub fn get_value_duration(&self, section: &str, option: &str) -> Option<Duration> {
    unsafe {
      let mut t: ll::Struct_GNUNET_TIME_Relative = uninitialized();
      let r = ll::GNUNET_CONFIGURATION_get_value_time(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          section.as_ptr() as *const c_char,
          option.as_ptr() as *const c_char,
          &mut t);
      match r {
        ll::GNUNET_OK => t.rel_value_us.to_i64().map(Duration::microseconds),
        _             => None,
      }
    }
  }

  /// Get a file size from the config in the form of a `u64`. `None` is returned if the `section`
  /// or `option` are not found or if they don't index a file size.
  pub fn get_value_size(&self, section: &str, option: &str) -> Option<u64> {
    unsafe {
      let mut s: u64= uninitialized();
      let r = ll::GNUNET_CONFIGURATION_get_value_size(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          section.as_ptr() as *const c_char,
          option.as_ptr() as *const c_char,
          &mut s);
      match r {
        ll::GNUNET_OK => Some(s),
        _             => None,
      }
    }
  }
  
  /// Get a string from the config in the form of `String`. `None` is returned if the `section` or
  /// `option` are not found or if they don't index a string.
  pub fn get_value_string(&self, section: &str, option: &str) -> Option<String> {
    unsafe {
      let mut s: *mut c_char = ptr::null::<c_char>() as *mut c_char;
      let r = ll::GNUNET_CONFIGURATION_get_value_string(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          section.as_ptr() as *const c_char,
          option.as_ptr() as *const c_char,
          &mut s);
      let cs = s as *const c_char;
      let ret = match r {
        // TODO: config strings that aren't utf8 will will appear to not exist
        //       think of a better way to do this
        ll::GNUNET_OK => from_utf8(CStr::from_ptr(cs).to_bytes()).ok().map(|s| s.to_string()),
        _ => None,
      };
      free(s as *mut c_void);
      ret
    }
  }

  /// Get a choice value from the config. `choices` contains a list of possible choices one of
  /// which will be returned. `None` is returned if the `section` or `option` are not found or if
  /// they don't index a choice value or if the value is not one of the options given in `choices`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use gnunet::Configuration;
  ///
  /// let cfg = Configuration::default().unwrap();
  /// let s = cfg.get_value_choice("DHT", "CACHE_RESULTS", &["YES", "NO"]);
  /// assert!(s == Some("YES") || s == Some("NO") || s == None);
  /// ```
  pub fn get_value_choice<'a>(&self, section: &str, option: &str, choices: &[&'a str]) -> Option<&'a str> {
    unsafe {
      let c_choices = choices.iter()
                             .map(|s| s.as_bytes().as_ptr() as *const c_char)
                             .collect::<Vec<*const c_char>>();
      let mut s: *const c_char = ptr::null::<c_char>() as *const c_char;
      let r = ll::GNUNET_CONFIGURATION_get_value_choice(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          section.as_ptr() as *const c_char,
          option.as_ptr() as *const c_char,
          c_choices.as_ptr(),
          &mut s);
      match r {
        ll::GNUNET_OK => c_choices.iter()
                                  .zip(choices.iter())
                                  .find(|&(&cstr, _)| cstr == s as *const c_char)
                                  .map(|t| *t.1),
        _             => None,
      }
    }
  }

  /// Get a filename value from the config in the form of a `PathBuf`. `None` is returned if the
  /// `section` or `option` are not found or if they don't index a filename.
  pub fn get_value_filename(&self, section: &str, option: &str) -> Option<PathBuf> {
    unsafe {
      let mut s: *mut c_char = ptr::null::<c_char>() as *mut c_char;
      let r = ll::GNUNET_CONFIGURATION_get_value_filename(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          section.as_ptr() as *const c_char,
          option.as_ptr() as *const c_char,
          &mut s);
      let cs = s as *const c_char;
      let path = match from_utf8(CStr::from_ptr(cs).to_bytes()) {
        Ok(s)   => s,
        Err(_)  => return None,
      };
      let ret = match r {
        ll::GNUNET_OK => Some(PathBuf::from(path)),
        _             => None,
      };
      free(s as *mut c_void);
      ret
    }
  }

  /// Test whether the configuration options have been changed since the last
  /// save.
  pub fn is_dirty(&self) -> bool {
    unsafe {
      match ll::GNUNET_CONFIGURATION_is_dirty(self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle) {
        ll::GNUNET_NO => false,
        _             => true,
      }
    }
  }

  /// Save configuration to a file.
  pub fn save<P: AsPath + ?Sized>(&mut self, filename: &P) -> Result<(), ConfigurationSaveError> {
    let cpath = match to_c_path(filename) {
      Ok(cpath) => cpath,
      Err(e)    => return Err(ConfigurationSaveError::BadPath(e)),
    };
    let res = unsafe {
      ll::GNUNET_CONFIGURATION_write(self.data, cpath.as_ptr())
    };
    match res {
      ll::GNUNET_OK => Ok(()),
      _             => Err(ConfigurationSaveError::UnknownError),
    }
  }
}

/*
impl<'s> Index<&'s str, ConfigSection> for Configuration {
  fn index(&'a self, index: &&'s str) -> &'a ConfigSection {
    ConfigSection {
      conf: self.data,
      name: *index,
    }
  }
}
*/

impl FromStr for Configuration {
  type Err = ConfigurationFromStrError;

  fn from_str(s: &str) -> Result<Configuration, ConfigurationFromStrError> {
    let cfg = Configuration::empty();
    unsafe {
      match ll::GNUNET_CONFIGURATION_deserialize(cfg.data, s.as_ptr() as *const c_char, s.len() as size_t, 1) {
        ll::GNUNET_OK => Ok(cfg),
        _             => Err(ConfigurationFromStrError),
      }
    }
  }
}

impl ToString for Configuration {
  fn to_string(&self) -> String {
    unsafe {
      let mut size: size_t = uninitialized();
      let serialized = ll::GNUNET_CONFIGURATION_serialize(self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle, &mut size);
      let constified = serialized as *const c_char;
      let bytes = CStr::from_ptr(constified).to_bytes();
      let ret = match from_utf8(bytes) {
        Ok(s)   => s.to_string(),
        Err(_)  => panic!("GNUNET_CONFIGURATION_serialize returned invalid utf-8"),
      };
      free(serialized as *mut c_void);
      ret
    }
  }
}

impl Clone for Configuration {
  fn clone(&self) -> Configuration {
    Configuration {
      data: unsafe {
        ll::GNUNET_CONFIGURATION_dup(self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle)
      },
    }
  }
}

impl Drop for Configuration {
  fn drop(&mut self) {
    unsafe {
      ll::GNUNET_CONFIGURATION_destroy(self.data);
    }
  }
}

/*
impl<'s> Index<&'s str, ConfigValue> for ConfigSection {
  fn index(&'a self, index: &&'s str) -> &'a ConfigValue {

  }
}
*/

#[test]
fn test() {
  let cfg = Configuration::default();
  let _ = cfg.clone();
}

