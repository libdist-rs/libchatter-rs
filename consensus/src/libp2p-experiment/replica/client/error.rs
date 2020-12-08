use std::io;

#[derive(Debug)]
pub enum Error {
    ConnectFailed,
    ConnectionClosed,
    TxReadFailed(io::Error),
    BlockWriteFailed(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectFailed => write!(f, "Connection Failed"),
            Error::ConnectionClosed => write!(f, "Connection Closed"),
            Error::TxReadFailed(e) => write!(f, "Failed to read and parse transactions with error[{}]", e),
            Error::BlockWriteFailed(e) => write!(f, "Failed to parse/write the block to the client with error [{}]", e),
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
            Error::BlockWriteFailed(_) => "Failed to write block",
            Error::TxReadFailed(_) => "Failed to read TX",
        }
    }
}

