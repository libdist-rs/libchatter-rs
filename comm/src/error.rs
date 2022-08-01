#[derive(Debug)]
pub enum NetError {
    Generic(String),
    GenericStr(&'static str),
    IoErr(String),
}

impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NetError")
    }
}

impl std::error::Error for NetError {
    
}

impl std::convert::From<String> for NetError {
    fn from(str: String) -> Self {
        NetError::Generic(str)
    }
}

impl std::convert::From<&'static str> for NetError {
    fn from(str: &'static str) -> Self {
        NetError::GenericStr(str)
    }
}

impl std::convert::From<std::io::Error> for NetError {
    fn from(io_err: std::io::Error) -> Self {
        NetError::IoErr(io_err.to_string())
    }
}