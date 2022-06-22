#[derive(Debug)]
pub enum NetError {
    Generic(String),
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