use paths;
use regex::Regex;
use std;
use std::borrow::{Borrow, Cow};
use std::collections::{hash_map, HashMap};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::num::{ParseFloatError, ParseIntError};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use time;
use util;

pub struct Cfg {
    data: HashMap<String, HashMap<String, String>>,
}

quick_error! {
    #[derive(Debug)]
    pub enum ToolchainError {
        InvalidToolchainName ( name: String ) {
            display("invalid toolchain name: {}", name)
        }

        UnknownToolchainVersion { version: String } {
            display("unknown toolchain version: {}", version)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CfgDefaultError {
        NoDataDir {
            display("Failed to determine GNUnet installation data directory")
        }

        ReadDataDir { cause: io::Error } {
            display("Failed to read Gnunet installation data directory. Reason: {}", cause)
            from(cause: io::Error) -> { cause: cause }
            cause(cause)
        }

        LoadFile { cause: CfgLoadRawError } {
            display("Failed to load config file. Reason: {}", cause)
            cause(cause)
            from(cause: CfgLoadRawError) -> { cause: cause }
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CfgLoadRawError {
        FileOpen { cause: io::Error } {
            cause(cause)
            from(cause: io::Error) -> { cause: cause }
            display("Failed to open file. Reason: {}", cause)
        }

        Deserialize { cause: CfgDeserializeError } {
            cause(cause)
            from(cause: CfgDeserializeError) -> { cause: cause }
            display("Failed to deserialize config. Reason: {}", cause)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CfgDeserializeError {
        Io { cause: io::Error }{
            display("I/O error reading from reader. Specifically: {}", cause)
            from(cause: io::Error) -> { cause: cause }
            cause(cause)
        }

        LoadInline { cause: Box<CfgLoadRawError>, line_number: usize, filename: String }{
            display("Failed to load inline configuration file. line {}: Failed to load \"{}\" ({})", line_number, filename, cause)
        }

        InlineDisabled { line_number: usize, filename: String } {
            display("@INLINE@ directive in config but allow_inline is disabled. line {}: Will not load file \"{}\"", line_number, filename)
        }

        Syntax { line_number: usize, line: String } {
            display("Syntax error in configuration. line {}: Failed to parse \"{}\"", line_number, line)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CfgLoadError {
        LoadDefault { cause: CfgDefaultError } {
            display("Failed to load system default configuration. Reason: {}", cause)
            cause(cause)
            from(cause: CfgDefaultError) -> { cause: cause }
        }

        LoadFile { cause: CfgLoadRawError } {
            display("Failed to load the config file. Reason: {}", cause)
            cause(cause)
            from(cause: CfgLoadRawError) -> { cause: cause }
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CfgGetIntError {
        NoSection {
            display("The config does not contain a section with that name")
        }

        NoKey {
            display("The config section does contain that key")
        }

        Parse { cause: ParseIntError } {
            cause(cause)
            display("The value is not a valid u64. Details: {}", cause)
            from(cause: ParseIntError) -> { cause: cause }
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CfgGetFloatError {
        NoSection {
            display("The config does not contain a section with that name")
        }

        NoKey {
            display("The config section does contain that key")
        }

        Parse { cause: ParseFloatError } {
            display("The value is not a valid f32. Details: {}", cause)
            cause(cause)
            from(cause: ParseFloatError) -> { cause: cause }
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CfgGetRelativeTimeError {
        NoSection {
            display("The config does not contain a section with that name")
        }

        NoKey {
            display("The config section does contain that key")
        }

        Parse { cause: util::strings::ParseQuantityWithUnitsError } {
            cause(cause)
            display("The value is not a valid relative time. Reason: {}", cause)
            from(cause: util::strings::ParseQuantityWithUnitsError) -> { cause: cause }
        }
    }
}
quick_error! {
    #[derive(Debug)]
    pub enum CfgGetFilenameError {
        NoSection {
            display("The config does not contain a section with that name")
        }

        NoKey {
            display("The config section does contain that key")
        }

        ExpandDollar { cause: CfgExpandDollarError } {
            display("Failed to '$'-expand the config entry. Reason: {}", cause)
            cause(cause)
            from(cause: CfgExpandDollarError) -> { cause: cause }
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CfgExpandDollarError {
        NonUnicodeEnvVar { var_name: String } {
            display("Tried to expand to an environment variable containing invalid unicode. variable: \"{}\"", var_name)
        }

        Syntax { pos: usize } {
            display("Syntax error in '$'-expansion. Error at byte position {}", pos)
        }

        UnknownVariable { var_name: String } {
            display("Failed to expand variable. Variable not found in PATHS section or process environment: {}", var_name)
        }

        UnclosedBraces {
            display("'$'-expansion includes an unclosed '{{'")
        }
    }
}

impl Cfg {
    pub fn empty() -> Cfg {
        Cfg {
            data: HashMap::new(),
        }
    }

    pub fn load_raw<P: AsRef<Path>>(path: P) -> Result<Cfg, CfgLoadRawError> {
        let f = try!(File::open(path));
        Ok(try!(Cfg::deserialize(f, true)))
    }

    pub fn deserialize<R: Read>(read: R, allow_inline: bool) -> Result<Cfg, CfgDeserializeError> {
        use self::CfgDeserializeError::*;

        lazy_static! {
            static ref RE_KEY_VALUE: Regex = Regex::new(r"^(.+)=(.*)$").unwrap();
            static ref RE_SECTION: Regex = Regex::new(r"^\[(.+)\]$").unwrap();
            static ref RE_INLINE: Regex = Regex::new(r"^(?i)@inline@ (.+)$").unwrap();
        }

        let mut cfg = Cfg::empty();
        let mut section = String::new();
        let br = BufReader::new(read);
        for (i, res_line) in br.lines().enumerate() {
            let line_num = i + 1;
            let line_buf = try!(res_line);

            {
                let line = line_buf.trim();

                // ignore empty lines
                if line.is_empty() {
                    continue;
                }

                // ignore comments
                if line.starts_with('#') || line.starts_with('%') {
                    continue;
                }

                if let Some(caps) = RE_INLINE.captures(line) {
                    let filename = caps.get(1).unwrap().as_str().trim(); // panic is logically impossible
                    if allow_inline {
                        let cfg_raw = match Cfg::load_raw(filename) {
                            Ok(cfg_raw) => cfg_raw,
                            Err(e) => {
                                return Err(LoadInline {
                                    cause: Box::new(e),
                                    line_number: line_num,
                                    filename: filename.to_string(),
                                })
                            }
                        };
                        cfg.merge(cfg_raw);
                    } else {
                        return Err(InlineDisabled {
                            line_number: line_num,
                            filename: filename.to_string(),
                        });
                    }
                    continue;
                }

                if let Some(caps) = RE_SECTION.captures(line) {
                    section = caps.get(1).unwrap().as_str().to_string(); // panic is logically impossible
                    continue;
                }

                if let Some(caps) = RE_KEY_VALUE.captures(line) {
                    let key = caps.get(1).unwrap().as_str().trim();
                    let value = caps.get(2).unwrap().as_str().trim();

                    /*
                     * TODO: Make this less yukk. There's a whole bunch of unnecessary allocation
                     * and copying happening here.
                     */
                    match cfg.data.entry(section.clone()) {
                        hash_map::Entry::Occupied(mut soe) => {
                            match soe.get_mut().entry(key.to_string()) {
                                hash_map::Entry::Occupied(mut koe) => {
                                    koe.insert(value.to_string());
                                }
                                hash_map::Entry::Vacant(kve) => {
                                    kve.insert(value.to_string());
                                }
                            }
                        }
                        hash_map::Entry::Vacant(sve) => {
                            let map = sve.insert(HashMap::new());
                            map.insert(key.to_string(), value.to_string());
                        }
                    }
                    continue;
                };
            };

            return Err(Syntax {
                line_number: line_num,
                line: line_buf,
            });
        }
        Ok(cfg)
    }

    pub fn merge(&mut self, mut other: Cfg) {
        for (k, mut v) in other.data.drain() {
            match self.data.entry(k) {
                hash_map::Entry::Occupied(oe) => {
                    let map = oe.into_mut();
                    for (k, v) in v.drain() {
                        map.insert(k, v);
                    }
                }
                hash_map::Entry::Vacant(ve) => {
                    ve.insert(v);
                }
            }
        }
    }

    pub fn default() -> Result<Cfg, CfgDefaultError> {
        use self::CfgDefaultError::*;

        let mut data_dir = match paths::data_dir() {
            Some(dd) => dd,
            None => return Err(NoDataDir),
        };

        data_dir.push("config.d");
        let mut cfg = Cfg::empty();
        let rd = match std::fs::read_dir(data_dir) {
            Ok(dirent) => dirent,
            Err(e) => return Err(ReadDataDir { cause: e }),
        };

        for res_dirent in rd {
            let dirent = match res_dirent {
                Ok(dirent) => dirent,
                Err(e) => return Err(ReadDataDir { cause: e }),
            };
            let path = dirent.path();
            if let Ok(file_type) = dirent.file_type() {
                if path.extension() == Some(OsStr::new("conf")) && file_type.is_file() {
                    let cfg_raw = try!(Cfg::load_raw(path));
                    cfg.merge(cfg_raw);
                }
            }
        }

        Ok(cfg)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Cfg, CfgLoadError> {
        let mut cfg = try!(Cfg::default());
        let cfg_raw = try!(Cfg::load_raw(path));
        cfg.merge(cfg_raw);
        Ok(cfg)
    }

    pub fn get_int(&self, section: &str, key: &str) -> Result<u64, CfgGetIntError> {
        use self::CfgGetIntError::*;

        match self.data.get(section) {
            Some(map) => match map.get(key) {
                Some(value) => Ok(try!(u64::from_str(value))),
                None => Err(NoKey),
            },
            None => Err(NoSection),
        }
    }

    pub fn get_float(&self, section: &str, key: &str) -> Result<f32, CfgGetFloatError> {
        use self::CfgGetFloatError::*;

        match self.data.get(section) {
            Some(map) => match map.get(key) {
                Some(value) => Ok(try!(f32::from_str(value))),
                None => Err(NoKey),
            },
            None => Err(NoSection),
        }
    }

    pub fn get_relative_time(
        &self,
        section: &str,
        key: &str,
    ) -> Result<time::Relative, CfgGetRelativeTimeError> {
        use self::CfgGetRelativeTimeError::*;

        match self.data.get(section) {
            Some(map) => match map.get(key) {
                Some(value) => Ok(try!(time::Relative::from_str(value))),
                None => Err(NoKey),
            },
            None => Err(NoSection),
        }
    }

    pub fn get_filename(&self, section: &str, key: &str) -> Result<PathBuf, CfgGetFilenameError> {
        use self::CfgGetFilenameError::*;

        match self.data.get(section) {
            Some(map) => match map.get(key) {
                Some(value) => {
                    let expanded = try!(self.expand_dollar(value));
                    Ok(PathBuf::from(expanded))
                }
                None => Err(NoKey),
            },
            None => Err(NoSection),
        }
    }

    pub fn set_string<'a, S, K>(
        &mut self,
        section: Cow<'a, str>,
        key: Cow<'a, str>,
        mut value: String,
    ) -> Option<String> {
        if let Some(map) = self.data.get_mut(&*section) {
            if let Some(mut val) = map.get_mut(&*key) {
                std::mem::swap(val, &mut value);
                return Some(value);
            }
            map.insert(section.into_owned(), value);
            return None;
        }

        let mut map = HashMap::with_capacity(1);
        map.insert(key.into_owned(), value);
        self.data.insert(section.into_owned(), map);
        None
    }

    pub fn expand_dollar<'o>(&self, orig: &'o str) -> Result<String, CfgExpandDollarError> {
        use self::CfgExpandDollarError::*;

        let lookup = |name: &str| {
            use std::env::VarError;

            match self.data.get("PATHS").and_then(|m| m.get(name)) {
                Some(v) => Some(self.expand_dollar(v)),
                None => match std::env::var(name) {
                    Ok(s) => Some(self.expand_dollar(s.borrow())),
                    Err(e) => match e {
                        VarError::NotPresent => return None,
                        VarError::NotUnicode(_) => {
                            return Some(Err(NonUnicodeEnvVar {
                                var_name: name.to_string(),
                            }))
                        }
                    },
                },
            }
        };

        let mut ret = String::with_capacity(orig.len());
        let mut chars = orig.char_indices().peekable();

        while let Some((_, c)) = chars.next() {
            if c == '$' {
                if let Some(&(_, c)) = chars.peek() {
                    let get_name = |mut chars: std::iter::Peekable<std::str::CharIndices<'o>>| {
                        let start = match chars.peek() {
                            Some(&(start, _)) => start,
                            None => orig.len(),
                        };
                        loop {
                            if let Some(&(end, c)) = chars.peek() {
                                if !(c.is_alphanumeric() || c == '_') {
                                    let name = unsafe { orig.slice_unchecked(start, end) };
                                    return (name, chars);
                                }
                                chars.next();
                            } else {
                                let name = unsafe { orig.slice_unchecked(start, orig.len()) };
                                return (name, chars);
                            }
                        }
                    };
                    if c == '{' {
                        chars.next();
                        if let Some(&(start, _)) = chars.peek() {
                            let (name, nchars) = get_name(chars);
                            chars = nchars;
                            if name.is_empty() {
                                // got something like "${_" where _ is not alphanumeric
                                return Err(Syntax { pos: start });
                            }
                            if let Some((pos, c)) = chars.next() {
                                match c {
                                    '}' => match lookup(name) {
                                        Some(expanded) => ret.push_str(try!(expanded).borrow()),
                                        None => {
                                            return Err(UnknownVariable {
                                                var_name: name.to_string(),
                                            })
                                        }
                                    },
                                    ':' => {
                                        if let Some((pos, c)) = chars.next() {
                                            if c != '-' {
                                                return Err(Syntax { pos: pos });
                                            }
                                            if let Some(&(start, _)) = chars.peek() {
                                                let mut depth = 0usize;
                                                let end: usize;
                                                loop {
                                                    if let Some((e, c)) = chars.next() {
                                                        match c {
                                                            '{' => depth += 1,
                                                            '}' => {
                                                                if depth == 0 {
                                                                    end = e;
                                                                    break;
                                                                } else {
                                                                    depth -= 1;
                                                                }
                                                            }
                                                            _ => (),
                                                        }
                                                    } else {
                                                        return Err(UnclosedBraces);
                                                    }
                                                }
                                                if let Some(expanded) = lookup(name) {
                                                    // have "${name:-def}" and we were able to
                                                    // resolve `name` to `expanded`
                                                    ret.push_str(try!(expanded).borrow());
                                                } else {
                                                    // have "${name:-def}" and we were not able
                                                    // to resolve name
                                                    let def =
                                                        unsafe { orig.slice_unchecked(start, end) };
                                                    ret.push_str(
                                                        try!(self.expand_dollar(def)).borrow(),
                                                    );
                                                }
                                            } else {
                                                // string ended after "${name:-"
                                                return Err(UnclosedBraces);
                                            }
                                        } else {
                                            // string ended after "${name:"
                                            return Err(UnclosedBraces);
                                        }
                                    }
                                    _ => {
                                        // got string "${name_" where _ is an invalid character
                                        return Err(Syntax { pos: pos });
                                    }
                                }
                            } else {
                                return Err(UnclosedBraces);
                            }
                        } else {
                            return Err(UnclosedBraces);
                        }
                    } else {
                        let (name, nchars) = get_name(chars);
                        chars = nchars;
                        match lookup(name) {
                            Some(expanded) => ret.push_str(try!(expanded).borrow()),
                            None => {
                                return Err(UnknownVariable {
                                    var_name: name.to_string(),
                                })
                            }
                        }
                    }
                } else {
                    return Err(Syntax { pos: orig.len() });
                }
            } else {
                ret.push(c);
            }
        }
        return Ok(ret);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

    #[test]
    fn test_expand_dollar() {
        let mut cfg = Cfg::empty();

        let res = cfg.set_string("PATHS", "IN_PATHS", String::from("in_paths"));
        assert!(res.is_none());
        std::env::set_var("IN_ENV", "in_env");

        let unexpanded = "foo $IN_PATHS $IN_ENV ${NOT_ANYWHERE:-${IN_ENV}_wub}_blah";
        let expanded = unwrap_result!(cfg.expand_dollar(unexpanded));
        assert_eq!(expanded, "foo in_paths in_env in_env_wub_blah");
    }
}
