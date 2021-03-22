use async_trait::async_trait;

pub mod error;
pub mod gcs;
pub mod metadata;
pub mod sqlite;

use crate::error::Error;
use bytes::Bytes;

#[async_trait]
pub trait StorageClient: Send + Sync {
    async fn fetch_object(&self, name: &str) -> Result<Bytes, Error>;
    async fn create_object(&self, name: &str, bytes: Bytes) -> Result<(), Error>;
}
