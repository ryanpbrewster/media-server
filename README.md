# Rust GCS integration

Uses the `yup_oauth2` crate to navigate the GCP OAuth2 flow, then uses the
resulting AccessToken to make an HTTP GET request against GCS using the
`reqwest` crate.

Example usage:

```
cargo run --example=write -- \
  --creds=my-service-account.json \
  --bucket=my-example-bucket \
  --object=foo/bar/glowing_man.png \
  glowing_man.png
```


```
cargo run --example=read -- \
  --creds=my-service-account.json \
  --bucket=my-example-bucket \
  --object=foo/bar/glowing_man.png \
    > glowing_man.png
```
