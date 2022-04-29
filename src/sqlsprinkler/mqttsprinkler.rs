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
}

impl MqttSprinkler {
    /// Creates a new MqttSprinkler
    pub fn sprinkler(sprinkler_zone: Zone) -> MqttSprinkler {
        MqttSprinkler {
            name: format!("sqlsprinkler_zone_{}",sprinkler_zone.name),
            // Status topic should be in the format of sqlsprinkler/zone_id/status
            stat_t: format!("sqlsprinkler_zone_{}/status", sprinkler_zone.id),
            // Command topic should be in the frmat of sqlsprinkler/zone_id/command
            cmd_t: format!("sqlsprinkler_zone_{}/command", sprinkler_zone.id),
            uniq_id: format!("sqlsprinkler_zone_{}", sprinkler_zone.id),
        }
    }

    // Creates a MqttObject for the system toggle.
    pub fn system() -> MqttSprinkler {
        MqttSprinkler {
            name: String::from("sqlsprinkler_system_state"),
            stat_t: String::from("sqlsprinkler_system/status"),
            cmd_t: String::from("sqlsprinkler_system/command"),
            uniq_id: String::from("sqlsprinkler_system")
        }
    }
}