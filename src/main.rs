use bytes::Bytes;
use media_server::gcs::GcsClient;
use media_server::StorageClient;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use warp::http::StatusCode;
use warp::Filter;
use yup_oauth2::ServiceAccountAuthenticator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opts = Opts::from_args();
    let client = {
        let creds = yup_oauth2::read_service_account_key(opts.creds).await?;
        let auth = ServiceAccountAuthenticator::builder(creds).build().await?;
        let scopes = &["https://www.googleapis.com/auth/devstorage.read_write"];
        let token = auth.token(scopes).await?;
        Arc::new(GcsClient::new(token, opts.bucket))
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

    let fallback = warp::get().map(|| "No handler here, try /v0/o/{objectId}\n");
    let routes = hello.or(warp::path("v0").and(v0)).or(fallback);
    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
    Ok(())
}

async fn get_object(
    object_name: String,
    client: Arc<GcsClient>,
) -> Result<impl warp::Reply, Infallible> {
    match client.fetch_object(&object_name).await {
        Ok(obj) => Ok(warp::reply::with_status(obj.to_vec(), StatusCode::OK)),
        Err(err) => Ok(warp::reply::with_status(
            serde_json::to_vec(&err).unwrap(),
            err.status(),
        )),
    }
}
async fn create_object(
    bytes: Bytes,
    client: Arc<GcsClient>,
) -> Result<impl warp::Reply, Infallible> {
    let name = format!("u64-{}", rand::random::<u64>());
    match client.create_object(&name, bytes).await {
        Ok(_) => Ok(warp::reply::with_status(name, StatusCode::OK)),
        Err(err) => Ok(warp::reply::with_status(
            serde_json::to_string(&err).unwrap(),
            err.status(),
        )),
    }
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, parse(from_os_str))]
    creds: PathBuf,
    #[structopt(long)]
    bucket: String,
}
