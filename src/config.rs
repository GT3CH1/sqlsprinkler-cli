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

impl MyConfig {
    /// Set the sqlsprinkler username
    /// # Arguments
    /// * `username` - The username to set
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_sqlsprinkler_user("username");
    /// ```
    pub fn set_sqlsprinkler_user(&mut self, username: &str) {
        self.sqlsprinkler_user = username.to_string();
    }

    /// Set the sqlsprinkler password
    /// # Arguments
    /// * `password` - The password to set
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_sqlsprinkler_pass("password");
    /// ```
    pub fn set_sqlsprinkler_pass(&mut self, password: &str) {
        self.sqlsprinkler_pass = password.to_string();
    }

    /// Set the sqlsprinkler host
    /// # Arguments
    /// * `host` - The host of the database to connect to
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_sqlsprinkler_host("host");
    /// ```
    pub fn set_sqlsprinkler_host(&mut self, host: &str) {
        self.sqlsprinkler_host = host.to_string();
    }

    /// Set the sqlsprinkler database
    /// # Arguments
    /// * `db` - The database to connect to
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_sqlsprinkler_db("db");
    /// ```
    pub fn set_sqlsprinkler_db(&mut self, db: &str) {
        self.sqlsprinkler_db = db.to_string();
    }

    /// Set the mqtt host
    /// # Arguments
    /// * `host` - The host of the mqtt server
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_mqtt_host("host");
    /// ```
    pub fn set_mqtt_host(&mut self, host: &str) {
        self.mqtt_host = host.to_string();
    }

    /// Set the mqtt username
    /// # Arguments
    /// * `username` - The username to set
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_mqtt_user("username");
    /// ```
    pub fn set_mqtt_user(&mut self, username: &str) {
        self.mqtt_user = username.to_string();
    }

    /// Set the mqtt password
    /// # Arguments
    /// * `password` - The password to set
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_mqtt_pass("password");
    /// ```
    pub fn set_mqtt_pass(&mut self, password: &str) {
        self.mqtt_pass = password.to_string();
    }

    /// Sets whether or not the mqtt client should be used
    /// # Arguments
    /// * `use_mqtt` - Whether or not to use the mqtt client
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_use_mqtt(true);
    /// ```
    pub fn set_use_mqtt(&mut self, use_mqtt: bool) {
        self.mqtt_enabled = use_mqtt;
    }

    /// Sets whether or not the program will start in verbose mode
    /// # Arguments
    /// * `verbose` - Whether or not to start in verbose mode
    /// # Returns
    /// * `Self` - The current instance of the configuration
    /// # Example
    /// ```
    /// use sqlsprinkler::config::MyConfig;
    /// let mut config = MyConfig::default();
    /// config.set_verbose(true);
    /// ```
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }
}

/// Get the current configuration
pub fn get_settings() -> MyConfig {
    SETTINGS.read().unwrap().clone()
}

pub fn set_settings(settings: MyConfig) {
    *SETTINGS.write().unwrap() = settings;
}

/// Read the settings file from `/etc/sqlsprinlker/sqlsprinkler.conf` and load into memory.
pub fn read_settings() -> Result<(), confy::ConfyError> {
    let mut new_settings = SETTINGS.write().unwrap();
    *new_settings = confy::load_path(SETTINGS_FILE_PATH)?;
    Ok(())
}
