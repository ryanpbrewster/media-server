use reqwest::StatusCode;
use serde::{ser::SerializeStruct, Serialize};

#[derive(Debug)]
pub enum Error {
    Application {
        status_code: StatusCode,
        text: String,
    },
    Network(reqwest::Error),
    OAuth(yup_oauth2::Error),
    Unknown(Box<dyn std::error::Error>),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Just use the Debug representation.
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}
impl Error {
    pub fn status(&self) -> StatusCode {
        match self {
            Error::Application { status_code, .. } => *status_code,
            Error::Network(err) => err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Error::OAuth(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl From<yup_oauth2::Error> for Error {
    fn from(err: yup_oauth2::Error) -> Self {
        Error::OAuth(err)
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
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Unknown(Box::new(err))
    }
}
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("GcsError", 2)?;
        s.serialize_field("status_code", &self.status().as_u16())?;
        match self {
            Error::Network(_) => s.serialize_field("message", "network error")?,
            Error::Application { text, .. } => s.serialize_field("message", text)?,
            Error::OAuth(err) => s.serialize_field("message", &format!("{:?}", err))?,
            Error::Unknown(err) => s.serialize_field("message", &format!("{}", err))?,
        }

        s.end()
    }
}
