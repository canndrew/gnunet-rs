use std::time::Duration;
use std::str::FromStr;
use std::{u32, u64};
use util;

pub struct Relative {
    micros: u64,
}

static RELATIVE_UNITS: [(&'static str, u64); 17] = [
    ("us", 1 ),
    ("ms", 1000 ),
    ("s", 1000 * 1000),
    ("\"", 1000  * 1000),
    ("m", 60 * 1000  * 1000),
    ("min", 60 * 1000  * 1000),
    ("minutes", 60 * 1000  * 1000),
    ("'", 60 * 1000  * 1000),
    ("h", 60 * 60 * 1000  * 1000),
    ("d", 24 * 60 * 60 * 1000 * 1000),
    ("day", 24 * 60 * 60 * 1000 * 1000),
    ("days", 24 * 60 * 60 * 1000 * 1000),
    ("week", 7 * 24 * 60 * 60 * 1000 * 1000),
    ("weeks", 7 * 24 * 60 * 60 * 1000 * 1000),
    ("year", 31536000000000 /* year */ ),
    ("years", 31536000000000 /* year */ ),
    ("a", 31536000000000 /* year */ ),
];

impl FromStr for Relative {
    type Err = util::strings::ParseQuantityWithUnitsError;
    fn from_str(s: &str) -> Result<Relative, util::strings::ParseQuantityWithUnitsError> {
        let micros = try!(util::strings::parse_quantity_with_units(s, &RELATIVE_UNITS[..]));
        Ok(Relative {
            micros: micros,
        })
    }
}

impl From<Duration> for Relative {
    fn from(d: Duration) -> Relative {
        Relative {
            micros: d.as_secs().checked_mul(1000000)
                               .and_then(|n| n.checked_add(d.subsec_nanos() as u64))
                               .unwrap_or(u64::MAX),
        }
    }
}

impl From<Relative> for Duration {
    fn from(r: Relative) -> Duration {
        if r.micros == u64::MAX {
            Duration::new(u64::MAX, u32::MAX)
        }
        else {
            Duration::new(r.micros / 1000000, ((r.micros % 1000000) as u32) * 1000)
        }
    }
}

#[cfg(tests)]
mod test {
    #[test]
    pub fn from_str_works() {
        let r = Relative::from_str(" 3   min  10 s   ");
        assert_eq!(r.micros, 190_000_000);
    }

    #[test]
    #[should_panic]
    pub fn parse_invalid_unit() {
        Relative::from_str("3 balls").unwrap();
    }

    #[test]
    #[should_panic]
    pub fn parse_no_unit() {
        Relative::from_str("12").unwrap();
    }

    #[test]
    #[should_panic]
    pub fn parse_empty_string() {
        Relative::from_str("").unwrap();
    }

    #[test]
    #[should_panic]
    pub fn empty_string() {
        Relative::from_str("").unwrap();
    }

    #[test]
    #[should_panic]
    pub fn parse_no_coefficient() {
        Relative::from_str("days").unwrap();
    }
}

