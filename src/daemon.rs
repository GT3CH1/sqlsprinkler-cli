use crate::{get_pool, get_system_status, set_system};
use warp::{Filter, http};
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SysStatus {
    state: bool,
}

#[tokio::main]
pub(crate) async fn run() {
    let get_sys_status = warp::get()
        .and(warp::path("system"))
        .and(warp::path("state"))
        .and(warp::path::end())
        .and_then(get_sys_status);

    let set_sys_status = warp::post()
        .and(warp::path("system"))
        .and(warp::path("state"))
        .and(warp::path::end())
        .and(post_json())
        .and_then(set_sys_status);

    let routes = get_sys_status.or(set_sys_status);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn post_json() -> impl Filter<Extract=(SysStatus, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

async fn get_sys_status() -> Result<impl warp::Reply, warp::Rejection> {
    let value = get_system_status(get_pool());
    Ok(warp::reply::json(&value))
}

async fn set_sys_status(status: SysStatus) -> Result<impl warp::Reply, warp::Rejection> {
    set_system(get_pool(), status.state);
    Ok(warp::reply::with_status("Success", http::StatusCode::OK))
}