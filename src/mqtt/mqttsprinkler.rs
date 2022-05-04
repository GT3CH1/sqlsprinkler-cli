use crate::Zone;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MqttSprinkler {
    // the name of the sprinkler
    pub name: String,
    // the status topic of the sprinkler (the topic to publish to)
    pub stat_t: String,
    // the command topic of the sprinkler (the topic to subscribe to)
    pub cmd_t: String,
    // The unique id of the sprinkler
    pub uniq_id: String,
    pub ic: String,
}

impl MqttSprinkler {
    /// Creates a new MqttSprinkler
    pub fn sprinkler(sprinkler_zone: &Zone) -> MqttSprinkler {
        MqttSprinkler {
            name: format!("sqlsprinkler_zone_{}", sprinkler_zone.name),
            // Status topic should be in the format of sqlsprinkler/zone_id/status
            stat_t: format!("sqlsprinkler_zone_{}/status", sprinkler_zone.id),
            // Command topic should be in the frmat of sqlsprinkler/zone_id/command
            cmd_t: format!("sqlsprinkler_zone_{}/command", sprinkler_zone.id),
            uniq_id: format!("sqlsprinkler_zone_{}", sprinkler_zone.id),
            ic: "mdi:sprinkler-variant".to_string(),
        }
    }

    // Creates a MqttObject for the system toggle.
    pub fn system() -> MqttSprinkler {
        MqttSprinkler {
            name: String::from("sqlsprinkler_system_state"),
            stat_t: String::from("sqlsprinkler_system/status"),
            cmd_t: String::from("sqlsprinkler_system/command"),
            uniq_id: String::from("sqlsprinkler_system"),
            ic: "mdi:electric-switch".to_string(),
        }
    }

    pub fn zone_time(zone: &Zone) -> MqttSprinkler {
        MqttSprinkler {
            name: format!("sqlsprinkler_zone_{}_time", zone.name),
            stat_t: format!("sqlsprinkler_zone_{}_time/status", zone.id),
            cmd_t: format!("sqlsprinkler_zone_{}_time/command", zone.id),
            uniq_id: format!("sqlsprinkler_zone_{}_time", zone.id),
            ic: "mdi:timer".to_string(),
        }
    }

    pub fn zone_auto_off(zone: &Zone) -> MqttSprinkler {
        MqttSprinkler {
            name: format!("sqlsprinkler_zone_{}_auto_off_state", zone.name),
            stat_t: format!("sqlsprinkler_zone_{}_auto_off_state/status", zone.id),
            cmd_t: format!("sqlsprinkler_zone_{}_auto_off_state/command", zone.id),
            uniq_id: format!("sqlsprinkler_zone_{}_auto_off_state", zone.id),
            ic: "mdi:electric-switch".to_string(),
        }
    }

    pub fn zone_enabled(zone: &Zone) -> MqttSprinkler {
        MqttSprinkler {
            name: format!("sqlsprinkler_zone_{}_enabled_state", zone.name),
            stat_t: format!("sqlsprinkler_zone_{}_enabled_state/status", zone.id),
            cmd_t: format!("sqlsprinkler_zone_{}_enabled_state/command", zone.id),
            uniq_id: format!("sqlsprinkler_zone_{}_enabled_state", zone.id),
            ic: "mdi:electric-switch".to_string(),
        }
    }
}
