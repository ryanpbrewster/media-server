use bytes::Bytes;
use log::info;
use media_server::StorageClient;
use media_server::{gcs::GcsClient, sqlite::SqliteClient};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use warp::http::StatusCode;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opts = Opts::from_args();
    let client: Arc<dyn StorageClient> = match opts.storage {
        Storage::Gcs { creds, bucket } => {
            let creds = yup_oauth2::read_service_account_key(creds).await?;
            Arc::new(GcsClient::new(creds, bucket).await?)
        }
        Storage::Sqlite { file } => Arc::new(SqliteClient::new(file).unwrap()),
    };

    let hello = warp::get().and(warp::path::end()).map(|| "ok");
    let v0 = {
        let c1: Arc<dyn StorageClient> = client.clone();
        let get_object = warp::path!("o" / String)
            .and(warp::get())
            .and(warp::any().map(move || c1.clone()))
            .and_then(get_object);
        let create_object = warp::path("o")
            .and(warp::post())
            .and(warp::header::optional::<String>("content-type"))
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
    client: Arc<dyn StorageClient>,
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
    media_type: Option<String>,
    bytes: Bytes,
    client: Arc<dyn StorageClient>,
) -> Result<impl warp::Reply, Infallible> {
    info!("Content-Type = {:?}", media_type);
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
    #[structopt(subcommand)]
    storage: Storage,
}

#[derive(Debug, StructOpt)]
enum Storage {
    #[structopt(name = "gcs")]
    Gcs {
        #[structopt(long)]
        creds: PathBuf,
        #[structopt(long)]
        bucket: String,
    },
    #[structopt(name = "sqlite")]
    Sqlite {
        #[structopt(long)]
        file: PathBuf,
    },
}
