use crate::{get_pool, get_system_status, set_system};
use warp::{Filter, http};
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SysStatus {
    system_status: bool,
}

impl SysStatus {
    fn new() -> Self {
        SysStatus {
            system_status: true,
        }
    }
}


#[tokio::main]
pub(crate) async fn run() {
    let sys_stat_filter_obj = SysStatus::new();
    let sys_filter = warp::any().map(move || sys_stat_filter_obj.clone());
    let get_sys_status = warp::get()
        .and(warp::path("system"))
        .and(warp::path("state"))
        .and(warp::path::end())
        .and(sys_filter.clone())
        .and_then(get_sys_status);

    let set_sys_status = warp::put()
        .and(warp::path("system"))
        .and(warp::path("state"))
        .and(warp::path::end())
        .and(post_json())
        .and_then(set_sys_status);

    let routes = get_sys_status
        .or(set_sys_status);
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn post_json() -> impl Filter<Extract=(SysStatus, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

async fn get_sys_status(_status: SysStatus) -> Result<impl warp::Reply, warp::Rejection> {
    let value = SysStatus {
        system_status: get_system_status(get_pool())
    };
    Ok(warp::reply::json(&value))
}

async fn set_sys_status(_status: SysStatus) -> Result<impl warp::Reply, warp::Rejection> {
    set_system(get_pool(), _status.system_status);
    println!("{}", _status.system_status);
    Ok(warp::reply::with_status("Success", http::StatusCode::OK))
}