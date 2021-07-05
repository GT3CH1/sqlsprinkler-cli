use serde::{Serialize, Deserialize};
use std::convert::From;
use std::{thread, time};
use crate::{set_pin, get_pool};
use std::borrow::Borrow;

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

impl Clone for Zone {
    fn clone(&self) -> Self {
        Zone {
            name: self.name.clone(),
            gpio: self.gpio.clone(),
            time: self.time.clone(),
            enabled: self.enabled.clone(),
            auto_off: self.auto_off.clone(),
            system_order: self.system_order.clone(),
            id: self.id.clone()
        }
    }
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
    pub name: String,
    pub gpio: i8,
    pub time: i8,
    pub enabled: bool,
    pub auto_off: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneWithState {
    pub name: String,
    pub gpio: i8,
    pub time: i8,
    pub enabled: bool,
    pub auto_off: bool,
    pub system_order: i8,
    pub state: bool,
    pub id: i8,
}
#[derive(Clone)]
pub struct ZoneList {
    pub zones: Vec<Zone>,
}

/// "Runs" the given zone, by turning it on for its specified run-time. USES A NEW THREAD.
/// # Params
///     * `zone` The zone we want to run. Will not run if the `enabled` flag for the zone is false.
///     * `auto_off` Whether or not we want to automatically turn off the system -> true for yes, false for no.
pub fn run_zone(zone: Zone, auto_off: bool) {
    let _zone = zone.clone();
    if !_zone.enabled {
        println!("Zone is not enabled!");
        return;
    }
    set_pin_zone(_zone.borrow(), true);
    if auto_off {
        thread::spawn(move || {
            let run_time = time::Duration::from_secs((_zone.time * 60) as u64);
            println!("{}",run_time.as_secs());

            thread::sleep(run_time);
            set_pin_zone(_zone.borrow(), false);
        });
    }
}

/// # Params
///     * `pin` the GPIO pin we want to turn on.
///     * `auto_off` Whether or not we want to automatically turn off the system -> true for yes, false for no.
pub fn run_zone_pin(zone_id: i8) {
    //TODO: I swear this needs to be easier some how but I don't know how
    let zone_list = get_zones();
    let mut _zone: Zone = Zone {
        name: "".to_string(),
        gpio: 0,
        time: 0,
        enabled: false,
        auto_off: false,
        system_order: 0,
        id: 0,
    };
    for zone in zone_list.zones.iter() {
        if zone_id == zone.id {
            _zone = Zone::from(zone);
        }
    }
    if !_zone.enabled {
        println!("Zone is not enabled!");
        return;
    }
    set_pin_zone(_zone.borrow(), true);
    if _zone.auto_off {
        thread::spawn(move || {
            let run_time = time::Duration::from_secs((_zone.time * 60) as u64);
            thread::sleep(run_time);
            set_pin_zone(_zone.borrow(), false);
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
pub fn change_zone_ordering(order: i8, zone: Zone) {
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
pub(crate) fn get_zones() -> ZoneList {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let rows = conn
        .query("SELECT Name, Gpio, Time, Enabled, AutoOff, SystemOrder, ID from Zones ORDER BY SystemOrder")
        .unwrap();
    let mut zoneList: Vec<Zone> = vec![];
    for row in rows {
        let _row = row.unwrap();
        let zone = Zone {
            name: _row.get(0).unwrap(),
            gpio: _row.get(1).unwrap(),
            time: _row.get(2).unwrap(),
            enabled: _row.get(3).unwrap(),
            auto_off: _row.get(4).unwrap(),
            system_order: _row.get(5).unwrap(),
            id: _row.get(6).unwrap(),
        };
        zoneList.push(zone);
    }
    let list = ZoneList {
        zones: zoneList
    };
    return list;
}