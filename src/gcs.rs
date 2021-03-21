use bytes::Bytes;
use log::info;
use reqwest::StatusCode;
use serde::{ser::SerializeStruct, Serialize};
use std::time::Instant;
use yup_oauth2::AccessToken;

pub struct GcsClient {
    token: AccessToken,
    bucket: String,
    client: reqwest::Client,
}
pub enum Error {
    Network(reqwest::Error),
    Application {
        status_code: StatusCode,
        text: String,
    },
}
impl Error {
    pub fn status(&self) -> StatusCode {
        match self {
            Error::Network(err) => err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Error::Application { status_code, .. } => *status_code,
        }
    }
}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err)
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

impl GcsClient {
    pub fn new(token: AccessToken, bucket: String) -> GcsClient {
        GcsClient {
            token,
            bucket,
            client: reqwest::Client::new(),
        }
    }
    pub async fn fetch_object(&self, name: &str) -> Result<Bytes, Error> {
        let url = format!(
            "https://storage.googleapis.com/storage/v1/b/{}/o/{}?alt=media",
            self.bucket, name
        );
        let req = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token.as_str()))
            .build()?;

        let start = Instant::now();
        let resp = self.client.execute(req).await.unwrap();
        if !resp.status().is_success() {
            return Err(Error::Application {
                status_code: resp.status(),
                text: resp.text().await.unwrap(),
            });
        }

        let bytes = resp.bytes().await.unwrap();
        info!(
            "fetched {} bytes in {}ms",
            bytes.len(),
            start.elapsed().as_millis()
        );
        Ok(bytes)
    }

    pub async fn create_object(&self, name: &str, bytes: Bytes) -> Result<(), Error> {
        let url = format!(
            "https://storage.googleapis.com/upload/storage/v1/b/{}/o?name={}",
            self.bucket, name,
        );
        let size = bytes.len();
        let req = self
            .client
            .post(&url)
            .body(bytes)
            .header("Authorization", format!("Bearer {}", self.token.as_str()))
            .build()?;

        let start = Instant::now();
        let resp = self.client.execute(req).await?;
        if !resp.status().is_success() {
            return Err(Error::Application {
                status_code: resp.status(),
                text: resp.text().await.unwrap(),
            });
        }
        info!("{:?}", resp.status());
        info!(
            "uploaded {} bytes in {}ms",
            size,
            start.elapsed().as_millis()
        );
        Ok(())
    }
}
