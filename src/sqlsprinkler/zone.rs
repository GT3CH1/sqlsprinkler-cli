use crate::get_settings;
use crate::sqlsprinkler::get_pool;
use mysql::{params, Row};
use rppal::gpio::{Gpio, OutputPin};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::{fmt, thread, time};

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
    pub(self) fn get_gpio(&self) -> OutputPin {
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
        // Check if mqtt is enabled
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
    pub(self) fn get_name(&self) -> String {
        self.clone().name
    }

    /// Gets whether or not this zone is on
    /// # Return
    ///     `on` A bool representing whether or not this zone is on.
    pub(self) fn is_on(&self) -> bool {
        let gpio = self.get_gpio();
        gpio.is_set_low()
    }

    /// Turns the zone on for 12 seconds and then turn off.
    pub fn test(&self) {
        if get_settings().verbose {
            println!("Testing {}", self.name)
        }
        self.turn_on();
        let run_time = time::Duration::from_secs(12);
        thread::sleep(run_time);
        self.turn_off();
    }

    /// Runs this zone, and automatically turn it off if launched from another thread and if
    /// `auto_off` is set to true for this zone. Will run for `time` minutes
    pub fn run_async(&self) {
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
    pub fn run(&self) {
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
    pub fn update(&self, zone: Zone) -> bool {
        let query = get_pool().prepare("UPDATE Zones SET Name=:name, Gpio=:gpio, Time=:time, AutoOff=:autooff,Enabled=:enabled,SystemOrder=:so WHERE ID=:id").into_iter();
        let mut updated: bool = false;
        for mut stmt in query {
            updated = stmt
                .execute(params! {
                    "name" => &zone.name,
                    "gpio" => zone.gpio,
                    "time" => zone.time,
                    "autooff" => zone.auto_off,
                    "enabled" => zone.enabled,
                    "so" => zone.system_order,
                    "id" => self.id
                })
                .unwrap()
                .affected_rows()
                == 1;
        }
        updated
    }

    /// Gets a representation of this zone, but also with `is_on` as bool `state`
    /// # Return
    /// `zone_with_state` A ZoneWithState struct representing this zone and its current state.
    pub fn get_with_state(&self) -> ZoneWithState {
        ZoneWithState {
            name: self.get_name(),
            gpio: self.gpio,
            time: self.time,
            enabled: self.enabled,
            auto_off: self.auto_off,
            system_order: self.system_order,
            state: self.is_on(),
            id: self.id,
        }
    }

    /// Updates the order of this zone
    /// # Params
    ///     `order` An i8 representing the new ordering of this zone.
    pub fn set_order(&self, order: i8) {
        let mut updated_zone = self.clone();
        updated_zone.system_order = order;
        self.update(updated_zone);
    }
}

/// Clone this zone.
impl Clone for Zone {
    fn clone(&self) -> Self {
        Zone {
            name: self.name.clone(),
            gpio: self.gpio,
            time: self.time,
            enabled: self.enabled,
            auto_off: self.auto_off,
            system_order: self.system_order,
            id: self.id,
        }
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {} | {} | {}",
            self.name,
            self.gpio,
            self.time,
            self.enabled,
            self.auto_off,
            self.system_order,
            self.id
        )
    }
}

/// Converts a row from the MySQL database to a Zone struct.
impl From<Row> for Zone {
    fn from(row: Row) -> Self {
        if get_settings().verbose {
            println!("{:?}", row);
        }
        Zone {
            name: row.get(0).unwrap(),
            gpio: row.get(1).unwrap(),
            time: row.get(2).unwrap(),
            enabled: row.get::<_, _>(3).unwrap(),
            auto_off: row.get::<_, _>(4).unwrap(),
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

/// Gets a zone from the given id
/// # Params
///     * `zone_id` The id of the zone we want to get
/// # Return
///     * `Zone` The zone that corresponds to the given id.
pub fn get_zone_from_id(zone_id: i8) -> Zone {
    let mut pool = get_pool().get_conn().unwrap();
    let mut stmt = pool
        .prepare(
            "SELECT Name, Gpio, Time, Enabled, AutoOff, SystemOrder, ID from Zones WHERE ID = :zi",
        )
        .unwrap();
    let mut _zone = Zone::default();

    let mut rows = stmt.execute(params! {"zi" => zone_id}).unwrap();

    // Get the first row in rows
    if rows.affected_rows() < 1 {
        if get_settings().verbose {
            println!("Default zone on get_zone_from_id");
        }
        return _zone;
    }
    let row = rows.next().unwrap().unwrap();
    _zone = Zone::from(row);
    if get_settings().verbose {
        println!("{:?}", _zone);
    }
    _zone
}

/// Deletes the given zone
/// # Params
///     * `_zone` The zone we are deleting
/// # Return
///     * a bool representing if the deletion was successful (true) or not (false)
pub fn delete_zone(_zone: ZoneDelete) -> bool {
    let pool = get_pool();
    let query = "DELETE FROM `Zones` WHERE id = ?";
    if get_settings().verbose {
        println!("{}", query);
    }
    let result = match pool.prep_exec(query, (_zone.id,)) {
        Ok(..) => true,
        Err(..) => {
            if get_settings().verbose {
                println!("An error occurred while deleting ")
            }
            false
        }
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
    let query =
        "INSERT into `Zones` (`Name`, `Gpio`, `Time`, `AutoOff`, `Enabled`) VALUES ( ?,?,?,?,? )";

    if get_settings().verbose {
        println!("{}", query);
    }
    let result = match pool.prep_exec(
        query,
        (
            _zone.name,
            _zone.gpio,
            _zone.time,
            _zone.auto_off,
            _zone.enabled,
        ),
    ) {
        Ok(..) => true,
        Err(e) => {
            if get_settings().verbose {
                println!("An error occurred while creating a new zone: {}", e);
            }
            false
        }
    };
    result
}

/// Creates a default zone.
impl Default for Zone {
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
