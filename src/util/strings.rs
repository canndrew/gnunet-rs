use std::num::ParseIntError;
use std::str::FromStr;

quick_error! {
    #[derive(Debug)]
    pub enum ParseQuantityWithUnitsError {
        ParseInt { cause: ParseIntError } {
            display("Failed to parse a number. Specifically: {}", cause)
            cause(cause)
            from(cause: ParseIntError) -> { cause: cause }
        }

        EmptyString {
            display("Empty string given as argument")
        }

        MissingUnit {
            display("Missing unit on the final number")
        }

        NoSuchUnit { unit: String } {
            display("Unrecognized unit. \"{}\" is not a valid unit", unit)
        }
    }
}

pub fn parse_quantity_with_units<'a>(
    s: &'a str,
    units: &[(&str, u64)],
) -> Result<u64, ParseQuantityWithUnitsError> {
    use self::ParseQuantityWithUnitsError::*;

    if s.trim().is_empty() {
        return Err(EmptyString);
    }

    let mut result = 0;
    let mut iter = s.split(' ');
    loop {
        match iter.next() {
            None => return Ok(result),
            Some(amount_str) => {
                if amount_str.is_empty() {
                    continue;
                } else {
                    let amount = try!(u64::from_str(amount_str));
                    loop {
                        match iter.next() {
                            None => return Err(MissingUnit),
                            Some(unit) => {
                                if unit.is_empty() {
                                    continue;
                                } else {
                                    let mut found = false;
                                    for &(u, multiplier) in units.iter() {
                                        if u == unit {
                                            result += amount * multiplier;
                                            found = true;
                                            break;
                                        }
                                    }
                                    if found {
                                        break;
                                    } else {
                                        return Err(NoSuchUnit {
                                            unit: unit.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
