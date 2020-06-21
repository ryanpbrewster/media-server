use bytes::Bytes;
use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();

    let hello = warp::get().and(warp::path::end()).map(|| "ok");

    let v0 = {
        let get_object = warp::path!("o" / String)
            .and(warp::get())
            .map(|_name: String| "TODO");
        let create_object = warp::path("o")
            .and(warp::post())
            .and(warp::body::bytes())
            .map(|bytes: Bytes| format!("{} bytes", bytes.len()));

        get_object.or(create_object)
    };

    let routes = hello.or(warp::path("v0").and(v0));
    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
}
