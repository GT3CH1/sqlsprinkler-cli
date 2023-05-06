use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

const SETTINGS_FILE_PATH: &str = "/etc/sqlsprinkler/sqlsprinkler.conf";

lazy_static! {
    static ref SETTINGS: RwLock<MyConfig> = RwLock::new(MyConfig::default());
}

/// Configuration for the application
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Default)]
pub struct MyConfig {
    /// The user to connect to the database as
    pub sqlsprinkler_user: String,

    /// The password to connect to the database with
    pub sqlsprinkler_pass: String,

    /// The database host to connect to
    pub sqlsprinkler_host: String,

    /// The name of the database to connect to
    pub sqlsprinkler_db: String,

    /// Whether or not the application should be running in verbose mode.
    pub verbose: bool,
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
