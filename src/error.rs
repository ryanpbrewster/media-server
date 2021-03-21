use reqwest::StatusCode;
use serde::{ser::SerializeStruct, Serialize};

pub enum Error {
    Application {
        status_code: StatusCode,
        text: String,
    },
    Network(reqwest::Error),
}
impl Error {
    pub fn status(&self) -> StatusCode {
        match self {
            Error::Application { status_code, .. } => *status_code,
            Error::Network(err) => err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err)
    }
}
impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::Application {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            text: format!("{:?}", err),
        }
    }
}
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("GcsError", 2)?;
        s.serialize_field("status_code", &self.status().as_u16())?;
        s.serialize_field(
            "message",
            match self {
                Error::Network(_) => "network error",
                Error::Application { text, .. } => text,
            },
        )?;
        s.end()
    }
}
