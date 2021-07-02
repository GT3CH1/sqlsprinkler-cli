use crate::{get_pool, get_system_status, set_system, get_zones, zone};
use warp::{Filter, http};
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SysStatus {
    system_status: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ZoneList {
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

    // Handle get requests to /system/state
    let get_sys_status = warp::get()
        .and(warp::path("system"))
        .and(warp::path("state"))
        .and(warp::path::end())
        .and_then(get_sys_status);

    // Handle put requests to /system/state
    let set_sys_status = warp::put()
        .and(warp::path("system"))
        .and(warp::path("state"))
        .and(warp::path::end())
        .and(sys_status_post_json())
        .and_then(set_sys_status);

    let get_zone_status = warp::get()
        .and(warp::path("zone"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and_then(get_zone_status);

    let routes = get_sys_status
        .or(set_sys_status)
        .or(get_zone_status);
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

/// Used to filter a post request to change the system status
fn sys_status_post_json() -> impl Filter<Extract=(SysStatus, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

/// Gets the system status
/// # Returns
///     * `json` A json object representing the current state of the system schedule.
async fn get_sys_status() -> Result<impl warp::Reply, warp::Rejection> {
    let value = SysStatus {
        system_status: get_system_status(get_pool())
    };
    Ok(warp::reply::json(&value))
}

/// Sets the system status
/// # Params
///     * `_status` The SysStatus object containing the value we are going to set the system status to.
async fn set_sys_status(_status: SysStatus) -> Result<impl warp::Reply, warp::Rejection> {
    set_system(get_pool(), _status.system_status);
    println!("{}", _status.system_status);
    Ok(warp::reply::with_status("Success", http::StatusCode::OK))
}

/// Gets the status of all the zones.
async fn get_zone_status() -> Result<impl warp::Reply, warp::Rejection> {
    let zone_list: Vec<zone::Zone> = get_zones(get_pool());
    Ok(warp::reply::json(&zone_list))
}
