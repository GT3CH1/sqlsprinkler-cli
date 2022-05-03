use lazy_static::lazy_static;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

const SETTINGS_FILE_PATH: &str = "/etc/sqlsprinkler/sqlsprinkler.conf";

lazy_static! {
    static ref SETTINGS: RwLock<MyConfig> = RwLock::new(MyConfig::default());
}


#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct MyConfig {
    pub sqlsprinkler_user: String,
    pub sqlsprinkler_pass: String,
    pub sqlsprinkler_host: String,
    pub sqlsprinkler_db: String,
    pub mqtt_host: String,
    pub mqtt_user: String,
    pub mqtt_pass: String,
    pub mqtt_enabled: bool,
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

pub fn get_settings() -> MyConfig {
    SETTINGS.read().unwrap().clone()
}


/// Read the settings file from `/etc/sqlsprinlker/sqlsprinkler.conf` and load into memory.
pub fn read_settings() -> Result<(), confy::ConfyError> {
    let mut new_settings = SETTINGS.write().unwrap();
    *new_settings = confy::load_path(SETTINGS_FILE_PATH)?;
    Ok(())
}