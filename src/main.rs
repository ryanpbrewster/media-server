use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();

    let hello = warp::get()
        .and(warp::path::end())
        .map(|| format!("Hello, World!"));

    warp::serve(hello).run(([127, 0, 0, 1], 9000)).await;
}
