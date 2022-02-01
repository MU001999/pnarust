/// A type that represents either Success or Failure in kvs.
pub type Result<T> = std::result::Result<T, Error>;

/// A type that represents errors in kvs.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IOError: {0:?}")]
    IOError(#[from] std::io::Error),

    #[error("SerdeJSONError: {0:?}")]
    SerdeJSONError(#[from] serde_json::Error),

    #[error("SerdeError: {0:?}")]
    SerdeError(String),

    #[error("WalkdirError: {0:?}")]
    WalkdirError(#[from] walkdir::Error),

    #[error("SloggersError: {0:?}")]
    SloggersError(#[from] sloggers::Error),

    #[error("SledError: {0:?}")]
    SledError(#[from] sled::Error),

    #[error("Error Log Meet")]
    ErrorLogMeet,

    #[error("Key not found")]
    KeyNotFound,
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::SerdeError(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::SerdeError(msg.to_string())
    }
}
