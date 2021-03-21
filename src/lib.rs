use bytes::Bytes;
use log::info;
use std::time::Instant;
use yup_oauth2::AccessToken;

pub struct GcsClient {
    token: AccessToken,
    bucket: String,
    client: reqwest::Client,
}

impl GcsClient {
    pub fn new(token: AccessToken, bucket: String) -> GcsClient {
        GcsClient {
            token,
            bucket,
            client: reqwest::Client::new(),
        }
    }
    pub async fn fetch_object(&self, name: &str) -> Result<Bytes, String> {
        let url = format!(
            "https://storage.googleapis.com/storage/v1/b/{}/o/{}?alt=media",
            self.bucket, name
        );
        let req = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token.as_str()))
            .build()
            .unwrap();

        let start = Instant::now();
        let resp = self.client.execute(req).await.unwrap();
        if resp.status().is_success() {
            let bytes = resp.bytes().await.unwrap();
            info!(
                "fetched {} bytes in {}ms",
                bytes.len(),
                start.elapsed().as_millis()
            );
            Ok(bytes)
        } else {
            Err(resp.text().await.unwrap())
        }
    }

    pub async fn create_object(&self, name: &str, bytes: Bytes) {
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
            .build()
            .unwrap();

        let start = Instant::now();
        let resp = self.client.execute(req).await.unwrap();
        info!("{:?}", resp.status());
        info!(
            "uploaded {} bytes in {}ms",
            size,
            start.elapsed().as_millis()
        );
    }
}
