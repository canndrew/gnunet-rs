use std::ptr;
use std::path;
use libc::{c_char, c_void, free};
use std::c_str::CString;
use std::mem::uninitialized;
use std::time::Duration;

use ll;

pub struct Configuration {
  data: *mut ll::Struct_GNUNET_CONFIGURATION_Handle,
}

impl Configuration {
  pub fn default() -> Option<Configuration> {
    unsafe {
      let cfg = ll::GNUNET_CONFIGURATION_create();
      match ll::GNUNET_CONFIGURATION_load(cfg, ptr::null()) {
        ll::GNUNET_OK => Some(Configuration {
          data: cfg,
        }),
        _ => None,
      }
    }
  }

  pub fn load(filename: path::Path) -> Option<Configuration> {
    unsafe {
      let cfg = ll::GNUNET_CONFIGURATION_create();
      let r = filename.with_c_str(|path| {
        ll::GNUNET_CONFIGURATION_load(cfg, path)
      });
      match r {
        ll::GNUNET_OK => Some(Configuration {
          data: cfg,
        }),
        _ => None,
      }
    }
  }

  pub fn get_value_int(&self, section: &str, option: &str) -> Option<u64> {
    unsafe {
      let mut n: u64 = uninitialized();
      let c_section = section.to_c_str(); //TODO: check for NULs
      let c_option  = option.to_c_str();
      let r = ll::GNUNET_CONFIGURATION_get_value_number(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          c_section.as_ptr(),
          c_option.as_ptr(),
          &mut n);
      match r {
        ll::GNUNET_OK => Some(n),
        _             => None,
      }
    }
  }

  pub fn get_value_float(&self, section: &str, option: &str) -> Option<f32> {
    unsafe {
      let mut f: f32 = uninitialized();
      let c_section = section.to_c_str(); //TODO: check for NULs
      let c_option  = option.to_c_str();
      let r = ll::GNUNET_CONFIGURATION_get_value_float(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          c_section.as_ptr(),
          c_option.as_ptr(),
          &mut f);
      match r {
        ll::GNUNET_OK => Some(f),
        _             => None,
      }
    }
  }

  pub fn get_value_duration(&self, section: &str, option: &str) -> Option<Duration> {
    unsafe {
      let mut t: ll::Struct_GNUNET_TIME_Relative = uninitialized();
      let c_section = section.to_c_str(); //TODO: check for NULs
      let c_option  = option.to_c_str();
      let r = ll::GNUNET_CONFIGURATION_get_value_time(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          c_section.as_ptr(),
          c_option.as_ptr(),
          &mut t);
      match r {
        ll::GNUNET_OK => t.rel_value_us.to_i64().map(Duration::microseconds),
        _             => None,
      }
    }
  }

  pub fn get_value_size(&self, section: &str, option: &str) -> Option<u64> {
    unsafe {
      let mut s: u64= uninitialized();
      let c_section = section.to_c_str(); //TODO: check for NULs
      let c_option  = option.to_c_str();
      let r = ll::GNUNET_CONFIGURATION_get_value_size(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          c_section.as_ptr(),
          c_option.as_ptr(),
          &mut s);
      match r {
        ll::GNUNET_OK => Some(s),
        _             => None,
      }
    }
  }
  
  pub fn get_value_string(&self, section: &str, option: &str) -> Option<String> {
    unsafe {
      let mut s: *mut c_char = ptr::null::<c_char>() as *mut c_char;
      let c_section = section.to_c_str(); //TODO: check for NULs
      let c_option  = option.to_c_str();
      let r = ll::GNUNET_CONFIGURATION_get_value_string(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          c_section.as_ptr(),
          c_option.as_ptr(),
          &mut s);
      let ret = match r {
        //ll::GNUNET_OK => CString::new(s as *const c_char, false).as_str().map(|s| s.to_string()),
        ll::GNUNET_OK => CString::new(s as *const c_char).as_str().map(|s| s.to_string()),
        _ => None,
      };
      free(s as *mut c_void);
      ret
    }
  }

  pub fn get_value_choice<'a>(&self, section: &str, option: &str, choices: &[&'a str]) -> Option<&'a str> {
    unsafe {
      let c_choices = choices.iter()
                             .map(|s| s.as_bytes().as_ptr() as *const c_char)
                             .collect::<Vec<*const c_char>>();
      let mut s: *mut c_char = ptr::null::<c_char>() as *mut c_char;
      let c_section = section.to_c_str(); //TODO: check for NULs
      let c_option  = option.to_c_str();
      let r = ll::GNUNET_CONFIGURATION_get_value_string(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          c_section.as_ptr(),
          c_option.as_ptr(),
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

  pub fn get_value_filename(&self, section: &str, option: &str) -> Option<Path> {
    unsafe {
      let mut s: *mut c_char = ptr::null::<c_char>() as *mut c_char;
      let c_section = section.to_c_str(); //TODO: check for NULs
      let c_option  = option.to_c_str();
      let r = ll::GNUNET_CONFIGURATION_get_value_filename(
          self.data as *const ll::Struct_GNUNET_CONFIGURATION_Handle,
          c_section.as_ptr(),
          c_option.as_ptr(),
          &mut s);
      let ret = match r {
        //ll::GNUNET_OK => Path::new_opt(CString::new(s as *const c_char, false)),
        ll::GNUNET_OK => Path::new_opt(CString::new(s as *const c_char)),
        _ => None,
      };
      free(s as *mut c_void);
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

#[test]
fn test() {
  let cfg = Configuration::default();
  let cfg2 = cfg.clone();
}

