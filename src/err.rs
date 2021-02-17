use tokio::sync::mpsc::error::SendError;

use crate::bar::Update;

pub type Res<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AppError(String),
    DbusError(zbus::Error),
    DbusMessageError(zbus::MessageError),
    FmtError(std::fmt::Error),
    IoError(std::io::Error),
    StdNumError(std::num::ParseIntError),
    SwayipcError(swayipc_async::Error),
    TokioError(SendError<Update>),
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error::AppError(error.to_string())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::AppError(error)
    }
}

impl From<zbus::MessageError> for Error {
    fn from(error: zbus::MessageError) -> Self {
        Error::DbusMessageError(error)
    }
}

impl From<zbus::Error> for Error {
    fn from(error: zbus::Error) -> Self {
        Error::DbusError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(error: std::fmt::Error) -> Self {
        Error::FmtError(error)
    }
}

impl From<SendError<Update>> for Error {
    fn from(error: SendError<Update>) -> Self {
        Error::TokioError(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error::StdNumError(error)
    }
}

impl From<swayipc_async::Error> for Error {
    fn from(error: swayipc_async::Error) -> Self {
        Error::SwayipcError(error)
    }
}
