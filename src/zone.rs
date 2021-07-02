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