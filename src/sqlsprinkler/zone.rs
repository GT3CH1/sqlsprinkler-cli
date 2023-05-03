use crate::get_settings;
use crate::sqlsprinkler::get_pool;
use log::{debug, error, info};
use mysql::{params, Row};
use rppal::gpio::{Gpio, OutputPin};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::{fmt, process, thread, time};

/// Represents a SQLSprinkler zone.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Zone {
    pub name: String,
    pub gpio: u8,
    pub time: u64,
    pub enabled: bool,
    pub auto_off: bool,
    pub system_order: i8,
    pub id: i8,
}

impl Zone {
    /// Gets the gpio interface for this zone.
    /// # Return
    ///     `gpio` An OutputPin that we can use to turn the zone on or off
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// let gpio = zone.get_gpio();
    /// ```
    pub(self) fn get_gpio(&self) -> OutputPin {
        let gpio = Gpio::new();
        if gpio.is_err() {
            error!("Failed to get GPIO interface. Are you on a Raspberry Pi / running as root?");
            process::exit(1);
        }
        let mut pin = gpio.unwrap().get(self.gpio).unwrap().into_output();
        // tl;dr without this line, pins get reset immediately.
        pin.set_reset_on_drop(false);
        pin
    }

    /// Turns on this zone.
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// zone.turn_on();
    /// ```
    pub fn turn_on(&self) {
        info!("Turned on {}", self);
        let mut gpio = self.get_gpio();
        gpio.set_low();
    }

    /// Turns off this zone.
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// zone.turn_off();
    /// ```
    pub fn turn_off(&self) {
        let mut gpio = self.get_gpio();
        info!("Turned off {}", self);
        gpio.set_high();
    }

    /// Gets the name of this zone
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// let name = zone.get_name();
    /// ```
    pub(self) fn get_name(&self) -> String {
        self.clone().name
    }

    /// Gets whether or not this zone is on
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// let is_on = zone.is_on();
    /// ```
    /// # Return
    ///     `on` A bool representing whether or not this zone is on.
    pub(self) fn is_on(&self) -> bool {
        let gpio = self.get_gpio();
        gpio.is_set_low()
    }

    /// Turns the zone on for 12 seconds and then turn off.
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// zone.test();
    /// ```
    pub fn test(&self) {
        info!("Testing {}", self.name);
        self.turn_on();
        let run_time = time::Duration::from_secs(12);
        thread::sleep(run_time);
        self.turn_off();
    }

    /// Runs this zone, and automatically turn it off if launched from another thread and if
    /// `auto_off` is set to true for this zone. Will run for `time` minutes
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// zone.run_async();
    /// ```
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
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// zone.run();
    /// ```
    pub fn run(&self) {
        self.turn_on();
        let _zone = self.clone();
        let run_time = time::Duration::from_secs(_zone.time * 60);
        thread::sleep(run_time);
        _zone.turn_off();
    }

    /// Updates this zone to the given `zone` parameter.
    /// # Params
    ///     `zone` The zone to update to.
    /// # Return
    ///     `true` if the zone was updated, `false` otherwise.
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// let mut new_zone = zone.clone();
    /// new_zone.name = "New Name".to_string();
    /// zone.update(new_zone);
    /// ```
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
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// let zone_with_state = zone.get_with_state();
    /// ```
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
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// zone.set_order(1);
    /// ```
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

/// Formats the zone to be displayed as
/// `name | gpio | time | auto_off | enabled | system_order | id`
impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Name: {} | Gpio: {} | Time: {} | Enabled: {} | AutoOff: {} | Order: {} | Id: {}",
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
        Zone {
            name: row.get("Name").unwrap(),
            gpio: row.get("GPIO").unwrap(),
            time: row.get("Time").unwrap(),
            enabled: row.get("Enabled").unwrap(),
            auto_off: row.get("AutoOff").unwrap(),
            system_order: row.get("SystemOrder").unwrap(),
            id: row.get("ID").unwrap(),
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

/// Used for reordering zones.
#[derive(Clone)]
pub struct ZoneList {
    pub zones: Vec<Zone>,
}

/// Gets a zone from the given id
/// # Params
///     `zone_id` The id of the zone we want to get
/// # Return
///     `Zone` The zone that corresponds to the given id.
/// # Example
/// ```
/// use sqlsprinkler::zone::Zone;
/// let zone = Zone::get_zone(1);
/// ```
pub fn get_zone_from_id(zone_id: i8) -> Zone {
    let query = format!(
        "SELECT Name,GPIO,Time,Enabled,AutoOff,SystemOrder,ID FROM Zones WHERE ID = {}",
        zone_id
    );

    let mut rows = get_pool().prep_exec(query, ()).unwrap();
    // let mut rows = stmt.prep+((zone_id,)).unwrap();
    // Get the first row in rows
    info!("Getting row from id: {}", zone_id);

    let mut _zone = Zone::default();
    if !rows.more_results_exists() {
        if get_settings().verbose {
            println!("Default zone on get_zone_from_id: {}", zone_id);
            println!("SELECT * FROM Zones WHERE id = {}", zone_id);
        }
        return _zone;
    }
    let row = rows.next().unwrap().unwrap();
    Zone::from(row)
}

/// Gets a zone from the given id
/// # Params
///     `zone_id` The id of the zone we want to get
/// # Return
///     `Zone` The zone that corresponds to the given id.
/// # Example
/// ```
/// use sqlsprinkler::zone::Zone;
/// let zone = Zone::get_zone_from_order(1);
/// ```
pub fn get_zone_from_order(zone_order: i8) -> Zone {
    let query = format!(
        "SELECT Name,GPIO,Time,Enabled,AutoOff,SystemOrder,ID FROM Zones WHERE SystemOrder = {}",
        zone_order
    );

    let mut rows = get_pool().prep_exec(query, ()).unwrap();
    // let mut rows = stmt.prep+((zone_id,)).unwrap();
    // Get the first row in rows
    info!("Getting row from system order: {}", zone_order);

    let mut _zone = Zone::default();
    if !rows.more_results_exists() {
        info!("Default zone on get_zone_from_order: {}", zone_order);
        return _zone;
    }
    let row = rows.next().unwrap().unwrap();
    Zone::from(row)
}

/// Deletes the given zone
/// # Params
///     `_zone` The zone we are deleting
/// # Return
///     a bool representing if the deletion was successful (true) or not (false)
/// # Example
/// ```
/// use sqlsprinkler::zone::Zone;
/// let zone = Zone::get_zone(1);
/// zone.delete();
/// ```
pub fn delete(_zone: ZoneDelete) -> bool {
    let pool = get_pool();
    let query = "DELETE FROM `Zones` WHERE id = ?";
    if get_settings().verbose {
        println!("{}", query);
    }
    let result = match pool.prep_exec(query, (_zone.id, )) {
        Ok(..) => true,
        Err(..) => {
            error!("An error occurred while deleting ");
            false
        }
    };
    if result {
        info!("Zone deleted!");
    }
    result
}

/// Adds a new zone
/// # Params
///     `ZoneAdd` The zone we are adding
/// # Example
/// ```
/// use sqlsprinkler::zone::Zone;
/// let zone = Zone::ZoneAdd {
///     name: "Test Zone".to_string(),
///     gpio: 1,
///     time: 1,
///     enabled: true,
///     auto_off: true,
/// }
/// Zone::add(zone);
/// ```
pub fn add(_zone: ZoneAdd) -> bool {
    let pool = get_pool();
    let query =
        "INSERT into `Zones` (`Name`, `Gpio`, `Time`, `AutoOff`, `Enabled`) VALUES ( ?,?,?,?,? )";
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
            error!("An error occurred while creating a new zone: {}", e);
            false
        }
    };
    if result {
        info!("Zone created!");
    }
    result
}
