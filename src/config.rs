use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

const SETTINGS_FILE_PATH: &str = "/etc/sqlsprinkler/sqlsprinkler.conf";

lazy_static! {
    static ref SETTINGS: RwLock<MyConfig> = RwLock::new(MyConfig::default());
}

/// Configuration for the application
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct MyConfig {
    /// The user to connect to the database as
    pub sqlsprinkler_user: String,

    /// The password to connect to the database with
    pub sqlsprinkler_pass: String,

    /// The database host to connect to
    pub sqlsprinkler_host: String,

    /// The name of the database to connect to
    pub sqlsprinkler_db: String,

    /// The host of the mqtt broker.
    pub mqtt_host: String,

    /// The user to connect to the mqtt broker as
    pub mqtt_user: String,

    /// The password to connect to the mqtt broker with
    pub mqtt_pass: String,

    /// Whether or not mqtt should be enabled
    pub mqtt_enabled: bool,

    /// Whether or not the application should be running in verbose mode.
    pub verbose: bool,
}

impl Default for MyConfig {
    fn default() -> Self {
        Self {
            sqlsprinkler_user: "".to_string(),
            sqlsprinkler_pass: "".to_string(),
            sqlsprinkler_host: "".to_string(),
            sqlsprinkler_db: "".to_string(),
            mqtt_host: "".to_string(),
            mqtt_user: "".to_string(),
            mqtt_pass: "".to_string(),
            mqtt_enabled: false,
            verbose: false,
        }
    }
}

impl Clone for MyConfig {
    fn clone(&self) -> MyConfig {
        MyConfig {
            sqlsprinkler_user: self.sqlsprinkler_user.clone(),
            sqlsprinkler_pass: self.sqlsprinkler_pass.clone(),
            sqlsprinkler_host: self.sqlsprinkler_host.clone(),
            sqlsprinkler_db: self.sqlsprinkler_db.clone(),
            mqtt_host: self.mqtt_host.clone(),
            mqtt_user: self.mqtt_user.clone(),
            mqtt_pass: self.mqtt_pass.clone(),
            mqtt_enabled: self.mqtt_enabled,
            verbose: self.verbose,
        }
    }
}

/// Get the current configuration
pub fn get_settings() -> MyConfig {
    SETTINGS.read().unwrap().clone()
}

/// Read the settings file from `/etc/sqlsprinlker/sqlsprinkler.conf` and load into memory.
pub fn read_settings() -> Result<(), confy::ConfyError> {
    let mut new_settings = SETTINGS.write().unwrap();
    *new_settings = confy::load_path(SETTINGS_FILE_PATH)?;
    Ok(())
}
