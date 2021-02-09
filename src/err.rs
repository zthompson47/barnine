use crate::bar::Update;

pub type Res<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AppError(String),
    AsyncIoError(async_std::channel::SendError<Update>),
    IoError(std::io::Error),
    DbusError(zbus::Error),
    DbusMessageError(zbus::MessageError),
}

impl From<&str> for Error {
    fn from (error: &str) -> Self {
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

impl From<async_std::channel::SendError<Update>> for Error {
    fn from(error: async_std::channel::SendError<Update>) -> Self {
        Error::AsyncIoError(error)
    }
}
