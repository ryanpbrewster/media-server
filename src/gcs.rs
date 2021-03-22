use crate::{error::Error, StorageClient};
use async_trait::async_trait;
use bytes::Bytes;
use log::info;
use std::time::Instant;
use yup_oauth2::{AccessToken, ServiceAccountAuthenticator, ServiceAccountKey};

pub struct GcsClient {
    token: AccessToken,
    bucket: String,
    client: reqwest::Client,
}
impl GcsClient {
    pub async fn new(creds: ServiceAccountKey, bucket: String) -> Result<GcsClient, Error> {
        let auth = ServiceAccountAuthenticator::builder(creds).build().await?;
        let scopes = &["https://www.googleapis.com/auth/devstorage.read_write"];
        let token = auth.token(scopes).await?;
        Ok(GcsClient {
            token,
            bucket,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl StorageClient for GcsClient {
    async fn fetch_object(&self, name: &str) -> Result<Bytes, Error> {
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

    async fn create_object(&self, name: &str, bytes: Bytes) -> Result<(), Error> {
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
