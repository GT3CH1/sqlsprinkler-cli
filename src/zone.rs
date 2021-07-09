use serde::{Serialize, Deserialize};
use std::convert::From;
use std::{thread, time};
use crate::{get_pool};
use rppal::gpio::{Gpio, OutputPin};
use std::error::Error;

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

impl Zone {
    /// Gets the pin of this zone
    /// # Return
    ///     `pin` A u8 representing the gpio pin.
    fn get_pin(&self) -> u8 {
        return self.gpio as u8;
    }

    /// Gets the gpio interface for this zone.
    /// # Return
    ///     `gpio` An OutputPin that we can use to turn the zone on or off
    pub(self) fn get_gpio(&self) -> OutputPin {
        let pin = self.get_pin();
        let pin = match Gpio::new() {
            Ok(gpio) => gpio.get(pin),
            Err(gpio) => Err((gpio)),
        };
        pin.unwrap().into_output()
    }

    /// Turns on this zone.
    pub fn turn_on(&self) {
        self.get_gpio().set_low();
    }

    /// Turns off this zone.
    pub fn turn_off(&self) {}

    /// Gets the name of this zone
    pub fn get_name(&self) -> String {
        return self.clone().name;
    }

    /// Runs this zone, and automatically turn it off if launched from another thread and if
    /// `auto_off` is set to true for this zone. Will run for `time` minutes
    pub fn run_zone(&self) {
        self.turn_on();
        if self.auto_off {
            let _zone = self.clone();
            thread::spawn(move || {
                let run_time = time::Duration::from_secs((_zone.time * 60) as u64);
                thread::sleep(run_time);
                _zone.turn_off()
            });
        }
    }

    /// Updates this zone to the given `zone` parameter.
    /// # Params
    ///     * `zone` A zone struct representing the new values for this zone.
    pub fn update_zone(&self, zone: Zone) {
        let pool = get_pool();
        let query = format!("UPDATE Zones SET Name='{}', Gpio={}, Time={},AutoOff={},Enabled={},SystemOrder={} WHERE ID={}"
                            , zone.name, zone.gpio, zone.time, zone.auto_off, zone.enabled, zone.system_order, zone.id);
        println!("{}", query);
        pool.prep_exec(query, ());
    }

    /// Gets whether or not this zone is on
    /// # Return
    ///     `on` A bool representing whether or not this zone is on.
    pub fn is_on(&self) -> bool {
        return self.get_gpio().is_set_low();
    }

    /// Gets a representation of this zone, but also with `is_on` as bool `state`
    /// # Return
    /// `zone_with_state` A ZoneWithState struct representing this zone and its current state.
    pub fn get_zone_with_state(&self) -> ZoneWithState {
        let new_zone = ZoneWithState {
            name: self.get_name(),
            gpio: self.gpio,
            time: self.time,
            enabled: self.enabled,
            auto_off: self.auto_off,
            system_order: self.system_order,
            state: self.is_on(),
            id: self.id,
        };
        return new_zone;
    }

    /// Updates the order of this zone
    /// # Params
    ///     `order` An i8 representing the new ordering of this zone.
    pub fn set_order(&self, order: i8) {
        let mut updated_zone = self.clone();
        updated_zone.system_order = order;
        self.update_zone(updated_zone);
    }
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
            id: self.id.clone(),
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

/// Gets a zone from the given id
/// # Params
///     * `zone_id` The id of the zone we want to get
/// # Return
///     * `Zone` The zone that corresponds to the given id.
pub fn get_zone_from_id(zone_id: i8) -> Zone {
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
            break;
        }
    }
    return _zone;
}

/// Deletes the given zone
/// # Params
///     * `_zone` The zone we are deleting
pub fn delete_zone(_zone: ZoneDelete) {
    let pool = get_pool();
    let query = format!("DELETE FROM `Zones` WHERE id = {}", _zone.id);
    pool.prep_exec(query, ());
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