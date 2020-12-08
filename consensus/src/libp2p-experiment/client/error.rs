use std::io;

#[derive(Debug)]
pub enum Error {
    ConnectFailed,
    ConnectionClosed,
    BlockReadFailed(io::Error),
    TxWriteFailed(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectFailed => write!(f, "Connection Failed"),
            Error::ConnectionClosed => write!(f, "Connection Closed"),
            Error::BlockReadFailed(e) => write!(f, "Failed to read and parse blocks with error[{}]", e),
            Error::TxWriteFailed(e) => write!(f, "Failed to parse/write the transactions to the server with error [{}]", e),
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            _ => None,
        }
    }

    fn description(&self) -> &str {
        match self {
            Error::ConnectFailed => "Connection failed",
            Error::ConnectionClosed => "Connection closed",
            Error::TxWriteFailed(_) => "Failed to write tx",
            Error::BlockReadFailed(_) => "Failed to read block",
        }
    }
}

