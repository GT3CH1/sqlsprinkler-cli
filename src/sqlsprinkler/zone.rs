use crate::sqlsprinkler::get_pool;
use log::{error, info};
use rppal::gpio::{Gpio, OutputPin};
use serde::{Deserialize, Serialize};
use std::{fmt, process, thread, time};
use structopt::StructOpt;

/// Represents a SQLSprinkler zone.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct Zone {
    #[allow(non_snake_case)]
    pub Name: String,
    #[allow(non_snake_case)]
    pub GPIO: i8,
    #[allow(non_snake_case)]
    pub Time: i64,
    #[allow(non_snake_case)]
    pub Enabled: bool,
    #[allow(non_snake_case)]
    pub Autooff: bool,
    #[allow(non_snake_case)]
    pub SystemOrder: i8,
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
    pub(self) fn get_gpio(&self) -> Result<OutputPin, rppal::gpio::Error> {
        let gpio = Gpio::new();
        return match gpio {
            Ok(gpio) => {
                let mut pin = gpio.get(self.GPIO as u8).unwrap().into_output();
                pin.set_reset_on_drop(false);
                Ok(pin)
            }
            Err(e) => {
                error!("Failed to get GPIO interface. Are you on a Raspberry Pi / running as root?");
                Err(e)
            }
        };
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
        let mut gpio = self.get_gpio().unwrap();
        gpio.set_low();
    }

    /// Turns off this zone.
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// zone.turn_off();
    /// ```
    pub fn turn_off(&self) -> Result<(), rppal::gpio::Error> {
        match self.get_gpio() {
            Ok(mut gpio) => {
                info!("Turned off {}", self);
                gpio.set_high();
                Ok(())
            }
            Err(e) => {
                error!("Failed to get GPIO interface. Are you on a Raspberry Pi / running as root?");
                Err(e)
            }
        }
    }

    /// Gets the name of this zone
    /// # Example
    /// ```
    /// use sqlsprinkler::zone::Zone;
    /// let zone = Zone::default();
    /// let name = zone.get_name();
    /// ```
    pub(self) fn get_name(&self) -> String {
        self.clone().Name
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
        let gpio = self.get_gpio().unwrap();
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
        info!("Testing {}", self.Name);
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
        if self.Autooff {
            // Need to clone because we are moving into a new thread.
            let _zone = self.clone();
            thread::spawn(move || {
                let run_time = time::Duration::from_secs((_zone.Time * 60) as u64);
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
        let run_time = time::Duration::from_secs((_zone.Time * 60) as u64);
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
    pub async fn update(&self, zone: Zone) -> Result<bool, sqlx::Error> {
        // let query = get_pool().prepare("UPDATE Zones SET Name=?, Gpio=?, Time=?, AutoOff=?, Enabled=? ,SystemOrder=? WHERE ID=?").into_iter();
        let rows = sqlx::query!(
            "UPDATE Zones SET Name=?, GPIO=?, Time=?, Autooff=?, Enabled=? ,SystemOrder=? WHERE ID=?",
            zone.Name,
            zone.GPIO,
            zone.Time,
            zone.Autooff,
            zone.Enabled,
            zone.SystemOrder,
            self.id
        ).execute(&get_pool().await?).await?;
        info!("Updated zone with id {}", self.id);
        Ok(true)
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
            gpio: self.GPIO,
            time: self.Time,
            enabled: self.Enabled,
            auto_off: self.Autooff,
            system_order: self.SystemOrder,
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
    pub async fn set_order(&self, order: i8) {
        let mut updated_zone = self.clone();
        updated_zone.SystemOrder = order;
        match self.update(updated_zone).await {
            Ok(_) => info!("Updated order of zone with id {}", self.id),
            Err(e) => error!("Failed to update order of zone with id {}: {}", self.id, e),
        }
    }
}

/// Clone this zone.
impl Clone for Zone {
    fn clone(&self) -> Self {
        Zone {
            Name: self.Name.clone(),
            GPIO: self.GPIO,
            Time: self.Time,
            Enabled: self.Enabled,
            Autooff: self.Autooff,
            SystemOrder: self.SystemOrder,
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
            self.Name,
            self.GPIO,
            self.Time,
            self.Enabled,
            self.Autooff,
            self.SystemOrder,
            self.id
        )
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
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, StructOpt)]
pub struct ZoneAdd {
    pub name: String,
    pub gpio: i8,
    pub time: u64,
    #[structopt(parse(try_from_str))]
    pub enabled: bool,
    #[structopt(parse(try_from_str))]
    pub auto_off: bool,
}

/// Used when we want to get a zone with whether or not it is turned on.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneWithState {
    pub name: String,
    pub gpio: i8,
    pub time: i64,
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
pub async fn get_zone_from_id(zone_id: i8) -> Result<Zone, sqlx::Error> {
    let zones = sqlx::query_as::<_, Zone>("SELECT * FROM Zones WHERE id = ?")
        .bind(zone_id)
        .fetch_all(&get_pool().await?)
        .await?;
    info!("Getting row from id: {}", zone_id);
    if zones.is_empty() {
        return Ok(Zone::default());
    }
    Ok(zones[0].clone())
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
pub async fn get_zone_from_order(zone_order: i8) -> Result<Zone, sqlx::Error> {
    let query = sqlx::query_as::<_, Zone>("SELECT * FROM Zones WHERE SystemOrder = ?")
        .bind(zone_order)
        .fetch_all(&get_pool().await?)
        .await?;
    let mut _zone = Zone::default();
    if query.len() == 0 {
        info!("Default zone on get_zone_from_order: {}", zone_order);
        return Ok(_zone);
    }
    Ok(query[0].clone())
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
pub async fn delete(_zone: ZoneDelete) -> Result<bool, sqlx::Error> {
    let query = sqlx::query!("DELETE FROM `Zones` WHERE `ID` = ?", _zone.id)
        .execute(&get_pool().await?)
        .await;
    let res = match query {
        Ok(_) => {
            info!("Zone deleted!");
            true
        }
        Err(e) => {
            error!("Error deleting zone: {}", e);
            false
        }
    };
    Ok(res)
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
pub async fn add(_zone: ZoneAdd) -> Result<bool, sqlx::Error> {
    let pool = get_pool().await?;
    let query = sqlx::query!(
        "INSERT INTO `Zones` (Name,GPIO,Time,Enabled,AutoOff,SystemOrder) VALUES (?,?,?,?,?,?)",
        _zone.name,
        _zone.gpio,
        _zone.time,
        _zone.enabled,
        _zone.auto_off,
        1
    ).execute(&pool).await;
    let res = match query {
        Ok(_) => {
            info!("Zone added!");
            true
        }
        Err(e) => {
            error!("Error adding zone: {}", e);
            false
        }
    };
    Ok(res)
}
