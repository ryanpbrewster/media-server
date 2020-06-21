use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();

    let hello = warp::get().and(warp::path::end()).map(|| "ok");

    let v0 = {
        let object = warp::path!("o" / String);
        let get_object = object.and(warp::get()).map(|_name: String| "TODO");
        let create_object = object.and(warp::post()).map(|_name: String| "TODO");

        get_object.or(create_object)
    };

    let routes = hello.or(warp::path("v0").and(v0));
    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
}
