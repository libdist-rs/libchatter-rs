#[derive(Debug)]
pub enum ParseError {
    // actual, expected
    InvalidMapLen(usize, usize),
    // f, n
    IncorrectFaults(usize, usize),
    // r
    InvalidMapEntry(usize),
    // pk_size
    InvalidPkSize(usize),
    // sk_size
    InvalidSkSize(usize),
    // feature name that is not implemented
    Unimplemented(&'static str),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ParseError::InvalidMapLen(exp, actual) => 
            write!(f, "invalid map length: expected {}, got {}", exp, actual),
            ParseError::IncorrectFaults(fault, n) => 
            write!(f, "n > 2f not satisfied since {} !> 2x{}", n, fault),
            ParseError::InvalidMapEntry(r) => 
            write!(f, "invalid map entry for {} replica", r),
            ParseError::InvalidPkSize(s) => 
            write!(f, "invalid public key size ({})", s),
            ParseError::Unimplemented(feature) =>
            write!(f, "{} feature is not yet implemented", feature),
            ParseError::InvalidSkSize(s) =>
            write!(f, "invalid secret key size ({})", s),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::InvalidMapLen(_,_) => "invalid map length",
            ParseError::IncorrectFaults(_,_) => "incorrect f and n values",
            ParseError::InvalidMapEntry(_) => "incorrect map entry",
            ParseError::InvalidPkSize(_) => "invalid public key size",
            ParseError::InvalidSkSize(_) => "invalid secret key size",
            ParseError::Unimplemented(_) => "feature unimplemented",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            _ => None,
        }
    }
}