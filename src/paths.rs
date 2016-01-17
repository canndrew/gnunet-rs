use std::path::PathBuf;
use std::env;
use std::ffi::OsStr;
use std::fs;

/**
 * Find the path of an executable. Similar to the unix `where` command.
 */
pub fn binary_path<S: AsRef<OsStr>>(name: &S) -> Option<PathBuf> {
    if let Ok(p) = env::var("PATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for path in p.split(sep) {
            if let Ok(rd) = fs::read_dir(path) {
                for res_dirent in rd {
                    if let Ok(dirent) = res_dirent {
                        if dirent.file_name().as_os_str() == name.as_ref() {
                            return Some(dirent.path());
                        }
                    }
                }
            }
        }
    }
    None
}

/**
 * Try to determine the prefix in which GNUnet was installed (eg. /usr/local)
 */
pub fn prefix_dir() -> Option<PathBuf> {
    if let Ok(p) = env::var("GNUNET_PREFIX") {
        return Some(PathBuf::from(p));
    }

    if let Some(mut bp) = binary_path(&"gnunet-arm") {
        if bp.pop() && bp.pop() {
            return Some(bp);
        }
    }

    None


    /*
    #[cfg(linux)]
    {
        use libc::funcs::posix88::unistd::getpid;

        /*
         * TODO: adapt get_path_from_proc_maps
         * need to use this lib not libgnunetutil
        
        let maps = format!("/proc/{}/maps", getpid());
        if let Ok(f) = File::open(maps) {
            let mut br = BufReader::new(f);
            for res_line in br.lines() {
                if let Ok(line) = res_line {
                    let re = regex!(r"^[a-f\d]+-[a-f\d]+ [r-][w-][x-][p-] [a-f\d]+  .....");
                }
            }
        }
        */
    }
    */



}

/**
 * Return the directory where data is installed (eg. /usr/share/gnunet)
*/
pub fn data_dir() -> Option<PathBuf> {
    if let Some(mut prefix) = prefix_dir() {
        prefix.push("share/gnunet");
        Some(prefix)
    }
    else {
        None
    }
}

