use std::fmt;

pub struct CoordError {
    pub msg: String,
}

impl fmt::Display for CoordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl fmt::Debug for CoordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CoordError: {}", self.msg)
    }
}
