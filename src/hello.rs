use std::fmt;

#[derive(Debug)]
pub struct Hello;

impl fmt::Display for Hello {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Hello!")
  }
}

