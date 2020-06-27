use bytes::Bytes;
use log::info;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use structopt::StructOpt;
use warp::http::StatusCode;
use warp::Filter;
use yup_oauth2::AccessToken;

#[tokio::main]
async fn main() {
    env_logger::init();

    let opts = Opts::from_args();
    let client = {
        let creds = yup_oauth2::read_service_account_key(opts.creds)
            .await
            .unwrap();
        let auth = yup_oauth2::ServiceAccountAuthenticator::builder(creds)
            .build()
            .await
            .unwrap();
        let scopes = &["https://www.googleapis.com/auth/devstorage.read_write"];
        let token = auth.token(scopes).await.unwrap();
        Arc::new(GcsClient {
            token,
            bucket: opts.bucket,
            client: reqwest::Client::new(),
        })
    };

    let hello = warp::get().and(warp::path::end()).map(|| "ok");
    let v0 = {
        let c1 = client.clone();
        let get_object = warp::path!("o" / String)
            .and(warp::get())
            .and(warp::any().map(move || c1.clone()))
            .and_then(get_object);
        let create_object = warp::path("o")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(warp::any().map(move || client.clone()))
            .and_then(create_object);

        get_object.or(create_object)
    };

    let routes = hello.or(warp::path("v0").and(v0));
    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
}

async fn get_object(
    object_name: String,
    client: Arc<GcsClient>,
) -> Result<impl warp::Reply, Infallible> {
    match client.fetch_object(&object_name).await {
        Ok(obj) => Ok(warp::reply::with_status(obj.to_vec(), StatusCode::OK)),
        Err(err) => Ok(warp::reply::with_status(
            err.into_bytes(),
            StatusCode::NOT_FOUND,
        )),
    }
}
async fn create_object(
    bytes: Bytes,
    client: Arc<GcsClient>,
) -> Result<impl warp::Reply, Infallible> {
    let name = format!("u64-{}", rand::random::<u64>());
    client.create_object(&name, bytes).await;
    Ok(name)
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, parse(from_os_str))]
    creds: PathBuf,
    #[structopt(long)]
    bucket: String,
}

struct GcsClient {
    token: AccessToken,
    bucket: String,
    client: reqwest::Client,
}

impl GcsClient {
    async fn fetch_object(&self, name: &str) -> Result<Bytes, String> {
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

    async fn create_object(&self, name: &str, bytes: Bytes) {
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
