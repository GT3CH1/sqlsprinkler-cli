use serde::{Serialize, Deserialize};
use std::convert::From;
use std::{thread, time, fmt};
use rppal::gpio::{Gpio, OutputPin};
use mysql::Row;
use crate::sqlsprinkler::get_pool;
use crate::get_settings;

/// Represents a SQLSprinkler zone.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    pub gpio: u8,
    pub time: u64,
    pub enabled: bool,
    pub auto_off: bool,
    pub system_order: i8,
    pub id: i8,
}

// - turn_on -> Allows user to turn on this zone
// - turn_off -> Turns off this zone.
// - get_name -> Gets the name for this zone
// - run_zone -> Runs the zone based off its configuration.
// - update_zone -> Updates the zone to a new zone.
// - is_on -> Whether or not this zone is currently active.
// - get_zone_with_state -> Returns the zone, but with the `state` parameter as well (ZoneWithState)
// - set_order -> Sets the system ordering for the zone.
impl Zone {
    /// Gets the gpio interface for this zone.
    /// # Return
    ///     `gpio` An OutputPin that we can use to turn the zone on or off
    pub fn get_gpio(&self) -> OutputPin {
        let mut pin = Gpio::new().unwrap().get(self.gpio).unwrap().into_output();
        // tl;dr without this line, pins get reset immediately.
        pin.set_reset_on_drop(false);
        pin
    }
    /// Turns on this zone.
    pub fn turn_on(&self) {
        if get_settings().verbose {
            println!("Turned on {}", self);
        }
        let mut gpio = self.get_gpio();
        gpio.set_low();
    }

    /// Turns off this zone.
    pub fn turn_off(&self) {
        let mut gpio = self.get_gpio();
        if get_settings().verbose {
            println!("Turned off {}", self);
        }
        gpio.set_high();
    }

    /// Gets the name of this zone
    pub fn get_name(&self) -> String {
        self.clone().name
    }

    /// Turns the zone on for 12 seconds and then turn off.
    pub fn test_zone(&self) {
        if get_settings().verbose {
            println!("Testing {}",self.name)
        }
        self.turn_on();
        let run_time = time::Duration::from_secs(12);
        thread::sleep(run_time);
        self.turn_off();
    }

    /// Runs this zone, and automatically turn it off if launched from another thread and if
    /// `auto_off` is set to true for this zone. Will run for `time` minutes
    pub fn run_zone_threaded(&self) {
        self.turn_on();
        if self.auto_off {
            // Need to clone because we are moving into a new thread.
            let _zone = self.clone();
            thread::spawn(move || {
                let run_time = time::Duration::from_secs(_zone.time * 60);
                thread::sleep(run_time);
                _zone.turn_off();
            });
        }
    }

    /// Runs this zone in a blocking fashion.
    pub fn run_zone(&self) {
        self.turn_on();
        let _zone = self.clone();
        let run_time = time::Duration::from_secs(_zone.time * 60);
        thread::sleep(run_time);
        _zone.turn_off();
    }

    /// Updates this zone to the given `zone` parameter.
    /// # Params
    ///     * `zone` A zone struct representing the new values for this zone.
    /// # Return
    ///     * `ok` A bool representing whether or not the update was successful
    pub fn update_zone(&self, zone: Zone) -> bool {
        let pool = get_pool();
        let query = format!("UPDATE Zones SET Name='{}', Gpio={}, Time={},AutoOff={},Enabled={},SystemOrder={} WHERE ID={}"
                            , zone.name, zone.gpio, zone.time, zone.auto_off, zone.enabled, zone.system_order, zone.id);
        let result = match pool.prep_exec(query, ()) {
            Ok(..) => true,
            Err(..) => false
        };
        result
    }

    /// Gets whether or not this zone is on
    /// # Return
    ///     `on` A bool representing whether or not this zone is on.
    pub fn is_on(&self) -> bool {
        let gpio = self.get_gpio();
        gpio.is_set_low()
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
        new_zone
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

/// Clone this zone.
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

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //NOTE: Dear god why did I think this was a good format.
        write!(f, "{} | {} | {} | {} | {} | {} | {}", self.name, self.gpio, self.time, self.enabled, self.auto_off, self.system_order, self.id)
    }
}

/// Converts a row from the MySQL database to a Zone struct.
impl From<Row> for Zone {
    fn from(row: Row) -> Self {
        Zone {
            name: row.get(0).unwrap(),
            gpio: row.get(1).unwrap(),
            time: row.get(2).unwrap(),
            enabled: row.get(3).unwrap(),
            auto_off: row.get(4).unwrap(),
            system_order: row.get(5).unwrap(),
            id: row.get(6).unwrap(),
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

/// Object representing the ordering of a zone.
/// # Params
///     * `order` A JSON list representing the new system ordering.
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
    pub time: u64,
    pub enabled: bool,
    pub auto_off: bool,
}

/// Used when we want to get a zone with whether or not it is turned on.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneWithState {
    pub name: String,
    pub gpio: u8,
    pub time: u64,
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

impl ::std::default::Default for Zone {
    fn default() -> Zone {
        Zone {
            name: "".to_string(),
            gpio: 0,
            time: 0,
            enabled: false,
            auto_off: false,
            system_order: 0,
            id: 0,
        }
    }
}

/// Gets a zone from the given id
/// # Params
///     * `zone_id` The id of the zone we want to get
/// # Return
///     * `Zone` The zone that corresponds to the given id.
pub fn get_zone_from_order(zone_order: i8) -> Zone {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let query = format!("SELECT Name, Gpio, Time, Enabled, AutoOff, SystemOrder, ID from Zones WHERE SystemOrder={}", zone_order);
    if get_settings().verbose {
        println!("{}", query);
    }
    let rows = conn.query(query).unwrap();
    let mut _zone = Zone::default();
    for row in rows {
        let _row = row.unwrap();
        _zone = Zone::from(_row);
        return _zone;
    }
    return _zone;
}

/// Deletes the given zone
/// # Params
///     * `_zone` The zone we are deleting
/// # Return
///     * a bool representing if the deletion was successful (true) or not (false)
pub fn delete_zone(_zone: ZoneDelete) -> bool {
    let pool = get_pool();
    let query = format!("DELETE FROM `Zones` WHERE id = {}", _zone.id);
    if get_settings().verbose {
        println!("{}", query);
    }
    let result = match pool.prep_exec(query, ()) {
        Ok(..) => true,
        Err(..) => {
            if get_settings().verbose {
                println!("An error occurred while deleting ")
            }
        },
    };
    result
}

/// Adds a new zone
/// # Params
///     * `_zone` The new zone we want to add.
///     * `pool` The MySQL connection pool to use.
/// # Return
///     * A bool representing whether or not the insert was successful (true) or failed (false)
pub fn add_new_zone(_zone: ZoneAdd) -> bool {
    let pool = get_pool();
    let query = format!("INSERT into `Zones` (`Name`, `Gpio`, `Time`, `AutoOff`, `Enabled`) VALUES \
     ( '{}','{}','{}',{},{} )", _zone.name, _zone.gpio, _zone.time, _zone.auto_off, _zone.enabled);
    if get_settings().verbose {
        println!("{}", query);
    }
    let result = match pool.prep_exec(query, ()) {
        Ok(..) => true,
        Err(e) => {
            if get_settings().verbose {
                println!("An error occurred while creating a new zone: {}",e);
                false
            }
        },
    };
    result
}
