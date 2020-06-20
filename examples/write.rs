use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let opts = Opts::from_args();

    let token = {
        let creds = yup_oauth2::read_service_account_key(opts.creds)
            .await
            .unwrap();
        let auth = yup_oauth2::ServiceAccountAuthenticator::builder(creds)
            .build()
            .await
            .unwrap();
        let scopes = &["https://www.googleapis.com/auth/devstorage.read_write"];
        auth.token(scopes).await.unwrap()
    };

    let client = reqwest::Client::new();
    let url = format!(
        "https://storage.googleapis.com/upload/storage/v1/b/{}/o?name={}",
        opts.bucket, opts.object
    );
    let mut body: Vec<u8> = Vec::new();
    let mut fin = File::open(opts.file).unwrap();
    fin.read_to_end(&mut body).unwrap();
    let size = body.len();
    eprintln!("preparing to upload {} bytes to {}", size, url);
    let req = client
        .post(&url)
        .body(body)
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .build()
        .unwrap();

    let resp = {
        let start = Instant::now();
        let resp = client.execute(req).await.unwrap().bytes().await.unwrap();
        eprintln!(
            "uploaded {} bytes in {}ms",
            size,
            start.elapsed().as_millis()
        );
        resp
    };

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write(&resp).unwrap();
    stdout.flush().unwrap();
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, parse(from_os_str))]
    creds: PathBuf,
    #[structopt(long)]
    bucket: String,
    #[structopt(long)]
    object: String,
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}
