use crate::{get_pool, get_system_status, set_system, get_zones, zone, add_new_zone, delete_zone};
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

    // Handle get requests to /system/state -> Used to get the current state of the sys schedule
    let get_sys_status = warp::get()
        .and(warp::path("system"))
        .and(warp::path("state"))
        .and(warp::path::end())
        .and_then(get_sys_status);

    // Handle put requests to /system/state -> Used to update the current state of the sys schedule
    let set_sys_status = warp::put()
        .and(warp::path("system"))
        .and(warp::path("state"))
        .and(warp::path::end())
        .and(sys_status_put_json())
        .and_then(set_sys_status);

    // Handle get requests to /zone/info -> Used for getting the INFORMATION of all the zones.
    let get_zone_status = warp::get()
        .and(warp::path("zone"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and_then(get_zone_status);

    // Handle put requests to /zone -> Used for TOGGLING a zone.
    let set_zone_status = warp::put()
        .and(warp::path("zone"))
        .and(warp::path::end())
        .and(zone_status_put_json())
        .and_then(set_zone_status);

    // Handles post request to /zone -> Used for CREATING a new zone.
    let add_zone = warp::post()
        .and(warp::path("zone"))
        .and(warp::path::end())
        .and(zone_post_json())
        .and_then(_add_zone);

    // Handles delete requests to /zone -> Used to DELETE a zone.
    let delete_zone = warp::delete()
        .and(warp::path("zone"))
        .and(warp::path::end())
        .and(zone_delete_json())
        .and_then(_delete_zone);

    let routes = get_sys_status
        .or(set_sys_status)
        .or(get_zone_status)
        .or(set_zone_status)
        .or(add_zone)
        .or(delete_zone);
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

/// Used to filter a put request to change the system status
fn sys_status_put_json() -> impl Filter<Extract=(SysStatus, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

/// Used to filter a put request to toggle a specific zone.
fn zone_status_put_json() -> impl Filter<Extract=(zone::ZoneToggle, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

/// Used to filter a post request to add a new zone.
fn zone_post_json() -> impl Filter<Extract=(zone::ZoneAdd, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

/// Used to filter a delete request to delete a new zone.
fn zone_delete_json() -> impl Filter<Extract=(zone::ZoneDelete, ), Error=warp::Rejection> + Clone {
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

/// Sets the id of the zone to the given state -> IE, turning on a zone.
async fn set_zone_status(_zone: zone::ZoneToggle) -> Result<impl warp::Reply, warp::Rejection> {
    let id = _zone.id;
    let state = _zone.state;
    let mut gpio = -1;

    let zone_list = get_zones(get_pool());
    for zone in zone_list.iter() {
        if zone.id == id {
            gpio = zone.gpio;
            println!("Found zone with gpio {}", gpio);
            break;
        }
    }

    // If we did not find a gpio pin, we need to throw an error.
    if gpio == -1 {
        //TODO: Error somehow?
    }
    if state {
        //TODO: Turn on the GPIO pin
    } else {
        //TODO: Turn off the GPIO pin
    }
    Ok(warp::reply::with_status(format!("Setting {} to {}", id, state), http::StatusCode::OK))
}

/// Adds a new zone to the system
/// # Params
///     * `_zone` The new zone we are wanting to add to the system.
async fn _add_zone(_zone: zone::ZoneAdd) -> Result<impl warp::Reply, warp::Rejection> {
    add_new_zone(_zone);
    Ok(warp::reply::with_status("Adding zone", http::StatusCode::CREATED))
}

/// Deletes a zone
/// # Params
///     * `_zone` The zone we are wanting to delete.
async fn _delete_zone(_zone: zone::ZoneDelete) -> Result<impl warp::Reply, warp::Rejection> {
    delete_zone(_zone);
    Ok(warp::reply::with_status("Deleted zone", http::StatusCode::OK))
}