use crate::{get_system_status, set_system, zone, turn_off_all_zones};
use warp::{Filter, http, reject};
use serde::{Serialize, Deserialize};
use crate::zone::{get_zone_from_id};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SysStatus {
    system_enabled: bool,
}

#[derive(Debug)]
struct LengthMismatch;

impl reject::Reject for LengthMismatch {}

#[tokio::main]
pub(crate) async fn run() {

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

    //  Handles put requests to /zone -> Used to UPDATE a zone.
    let update_zone = warp::put()
        .and(warp::path("zone"))
        .and(warp::path("update"))
        .and(warp::path::end())
        .and(zone_json())
        .and_then(_update_zone);

    // Handles put requests to /zone/order -> Used to UPDATE the ordering of the system
    let update_order = warp::put()
        .and(warp::path("zone"))
        .and(warp::path("order"))
        .and(warp::path::end())
        .and(order_json())
        .and_then(_update_order);

    let routes = get_sys_status
        .or(set_sys_status)
        .or(get_zone_status)
        .or(set_zone_status)
        .or(add_zone)
        .or(delete_zone)
        .or(update_zone)
        .or(update_order);
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
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

/// Used to filter a put request to update a zone.
fn zone_json() -> impl Filter<Extract=(zone::Zone, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

/// Used to filter a put request to re-order the system
fn order_json() -> impl Filter<Extract=(zone::ZoneOrder, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}


/// Gets the system status
/// # Returns
///     * `json` A json object representing the current state of the system schedule.
async fn get_sys_status() -> Result<impl warp::Reply, warp::Rejection> {
    let value = SysStatus {
        system_enabled: get_system_status()
    };
    Ok(warp::reply::json(&value))
}

/// Sets the system status
/// # Params
///     * `_status` The SysStatus object containing the value we are going to set the system status to.
async fn set_sys_status(_status: SysStatus) -> Result<impl warp::Reply, warp::Rejection> {
    set_system(_status.system_enabled);
    println!("{}", _status.system_enabled);
    Ok(warp::reply::with_status("Success", http::StatusCode::OK))
}

/// Gets the status of all the zones.
async fn get_zone_status() -> Result<impl warp::Reply, warp::Rejection> {
    let zone_list = zone::get_zones().clone();
    let mut zone_status_list: Vec<zone::ZoneWithState> = Vec::new();
    for zone in zone_list.zones.iter() {
        let _zone = zone::Zone::from(zone);
        let zone_with_status = _zone.get_zone_with_state();
        zone_status_list.push(zone_with_status);
    }
    Ok(warp::reply::json(&zone_status_list))
}

/// Sets the id of the zone to the given state -> IE, turning on a zone.
async fn set_zone_status(_zone: zone::ZoneToggle) -> Result<impl warp::Reply, warp::Rejection> {
    let state = _zone.state;
    let zone = get_zone_from_id(_zone.id);
    if state {
        /*
        NOTE:
         Here we want to run the zone instead of just turning it on. This is because we are running
         inside the daemon, and we can use the "auto-off" feature to automatically turn off the zone
         if unattended.
         */

        // Ensure that all zones are off
        turn_off_all_zones();
        zone.run_zone();
    } else {
        zone.turn_off();
    }
    Ok(warp::reply::with_status("Ok", http::StatusCode::OK))
}

/// Adds a new zone to the system
/// # Params
///     * `_zone` The new zone we are wanting to add to the system.
async fn _add_zone(_zone: zone::ZoneAdd) -> Result<impl warp::Reply, warp::Rejection> {
    zone::add_new_zone(_zone);
    Ok(warp::reply::with_status("Adding zone", http::StatusCode::CREATED))
}

/// Deletes a zone
/// # Params
///     * `_zone` The zone we are wanting to delete.
async fn _delete_zone(_zone: zone::ZoneDelete) -> Result<impl warp::Reply, warp::Rejection> {
    zone::delete_zone(_zone);
    Ok(warp::reply::with_status("Deleted zone", http::StatusCode::OK))
}

/// Updates a zone
/// # Params
///     * `_zone` The zone we want to update.
async fn _update_zone(_zone: zone::Zone) -> Result<impl warp::Reply, warp::Rejection> {
    let zone = zone::get_zone_from_id(_zone.id);
    zone.update_zone(_zone);
    Ok(warp::reply::with_status("Updated zone", http::StatusCode::OK))
}

async fn _update_order(_order: zone::ZoneOrder) -> Result<impl warp::Reply, warp::Rejection> {
    let zone_list = zone::get_zones();
    let _zone_list = zone_list.clone();
    let mut counter = 0;
    if _zone_list.zones.len() == _order.order.len() {
        for zone in _zone_list.zones.iter() {
            let _zone = zone::Zone::from(zone);
            let new_order = _order.order.as_slice()[counter];
            _zone.set_order(new_order);
            counter = counter + 1;
        }
        Ok(warp::reply::with_status("ok", http::StatusCode::OK))
    } else {
        Err(warp::reject::custom(LengthMismatch))
    }
}