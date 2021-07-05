use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use std::convert::From;
use std::{thread, time};
use crate::{set_pin, get_pool};
use mysql::Pool;

type Zones = Vec<Zone>;

/// Represents a sprinkler system zone
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    pub gpio: i8,
    pub time: i8,
    pub enabled: bool,
    pub auto_off: bool,
    pub system_order: i8,
    pub id: i8,
}

impl From<&Zone> for Zone {
    fn from(item: &Zone) -> Self {
        Zone {
            name: item.name.clone(),
            gpio: item.gpio,
            time: item.time,
            enabled: item.enabled,
            auto_off: item.auto_off,
            system_order: item.system_order,
            id: item.id,
        }
    }
}

/// Object representing toggling the zone.
/// # Params
///     * `id` The ID of the zone as it pertains in the database
///     * `state` The state to set the GPIO pin (true for on, false for off)
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneToggle {
    pub id: i8,
    pub state: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneOrder {
    pub order: Vec<i8>,
}

/// Used when are deleting a new zone via api
/// # Params
///     *   `id` The ID in the database that we are going to delete
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneDelete {
    pub id: i8,
}

/// Used when we are creating a new zone from an api response.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneAdd {
    pub name:  String,
    pub gpio: i8,
    pub time: i8,
    pub enabled: bool,
    pub auto_off: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneWithState {
    pub name:  String,
    pub gpio: i8,
    pub time: i8,
    pub enabled: bool,
    pub auto_off: bool,
    pub system_order: i8,
    pub state: bool,
    pub id: i8,
}

/// "Runs" the given zone, by turning it on for its specified run-time. USES A NEW THREAD.
/// # Params
///     * `zone` The zone we want to run. Will not run if the `enabled` flag for the zone is false.
///     * `auto_off` Whether or not we want to automatically turn off the system -> true for yes, false for no.
pub fn run_zone(zone: &'static Zone, auto_off: bool) {
    let _zone = zone.clone();
    if !_zone.enabled {
        println!("Zone is not enabled!");
        return;
    }
    set_pin_zone(_zone, true);
    if auto_off {
        thread::spawn(move || {
            let run_time = time::Duration::from_secs((_zone.time * 60) as u64);
            thread::sleep(run_time);
            set_pin_zone(_zone, false);
        });
    }
}


/// Sets the given zones gpio to the state we want
/// # Params
///     * `zone` The zone we want to control
///     * `state` The state we want the pin to be at - true for on, false for off.
pub fn set_pin_zone(zone: &Zone, state: bool) {
    // Ensure all the pins are turned off.
    set_pin(zone.gpio as u8, state);
}

/// Updates a zone with the given id to the values contained in this new zone.
/// # Params
///     * `zone` The zone containing the same id, but new information.
///     * `pool` The MySQL connection pool to use.
pub fn update_zone(zone: Zone) {
    let pool = get_pool();
    let query = format!("UPDATE Zones SET Name='{}', Gpio={}, Time={},AutoOff={},Enabled={},SystemOrder={} WHERE ID={}"
                        , zone.name, zone.gpio, zone.time, zone.auto_off, zone.enabled, zone.system_order, zone.id);
    println!("{}", query);
    pool.prep_exec(query, ());
}

/// Deletes the given zone
/// # Params
///     * `_zone` The zone we are deleting
pub fn delete_zone(_zone: ZoneDelete) {
    let pool = get_pool();
    let query = format!("DELETE FROM `Zones` WHERE id = {}", _zone.id);
    pool.prep_exec(query, ());
}


/// Updates the system order of the given zone to the given order, and then updates it in the database
/// # Params
///     * `order` The number representing the order of the zone
///     * `zone` The zone we want to change the order of.
fn change_zone_ordering(order: i8, zone: Zone) {
    let new_zone_order = Zone {
        name: zone.name,
        gpio: zone.gpio,
        time: zone.time,
        enabled: zone.enabled,
        auto_off: zone.auto_off,
        system_order: order,
        id: zone.id,
    };
    update_zone(new_zone_order);
}

/// Adds a new zone
/// # Params
///     * `_zone` The new zone we want to add.
///     * `pool` The MySQL connection pool to use.
pub(crate) fn add_new_zone(_zone: ZoneAdd) {
    let pool = get_pool();
    let query = format!("INSERT into `Zones` (`Name`, `Gpio`, `Time`, `AutoOff`, `Enabled`) VALUES \
     ( '{}','{}','{}',{},{} )", _zone.name, _zone.gpio, _zone.time, _zone.auto_off, _zone.enabled);
    pool.prep_exec(query, ());
}

/// Gets a list of all the zones in this database
/// # Arguments
///     * `pool` The SQL connection pool to use to query for zones
/// # Returns
///     * `Vec<Zone>` A list of all the zones in the database.
pub(crate) fn get_zones() -> Vec<Zone> {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let rows = conn
        .query("SELECT Name, Gpio, Time, Enabled, AutoOff, SystemOrder, ID from Zones ORDER BY SystemOrder")
        .unwrap();
    let mut zone_list = vec![];
    for row in &rows {
        let zone = Zone {
            name: row[0],
            gpio: row[1],
            time: row[2],
            enabled: row[3],
            auto_off: row[4],
            system_order: row[5],
            id: row[6],
        };
        zone_list.push( zone);
    }
    return zone_list;
}