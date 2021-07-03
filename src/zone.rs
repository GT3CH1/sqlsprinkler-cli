use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

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

/// Object representing toggling the zone.
/// # Params
///     * `id` The ID of the zone as it pertains in the database
///     * `state` The state to set the GPIO pin (true for on, false for off)
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneToggle {
    pub id: i8,
    pub state: bool
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

#[derive(Clone)]
struct ZoneList {
    zone_list: Arc<RwLock<Zones>>
}

/// The list of zones from the database
impl ZoneList {
    fn new() -> Self {
        ZoneList {
            zone_list: Arc::new(RwLock::new(Vec::new())),
        }
    }
}