gcs:
  RUST_LOG=info cargo run -- gcs --bucket=rpb-dev.appspot.com --creds=$HOME/tmp/service-account.json

sqlite:
  RUST_LOG=info cargo run -- sqlite --file=$HOME/tmp/media-server.sqlite3
