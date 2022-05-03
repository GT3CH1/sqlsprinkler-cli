// Copyright 2021 Gavin Pease

mod sqlsprinkler;
mod mqtt;

use std::fmt::Debug;
use std::process::exit;
use std::str::FromStr;
use std::sync::RwLock;
use mysql::serde_json;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use sqlsprinkler::daemon;
use crate::sqlsprinkler::zone::Zone;
use crate::sqlsprinkler::system::{get_system_status, get_zones, set_system_status, turn_off_all_zones, winterize};
use crate::sqlsprinkler::mqttsprinkler;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, StructOpt)]
#[structopt(name = "sqlsprinkler", about = "SQLSprinkler")]
pub struct Opts {
    #[structopt(short = "v", about = "Prints the version of SQLSprinkler.")]
    version_mode: bool,

    #[structopt(short = "w", long = "daemon", about = "Launches the SQLSprinkler API web daemon.")]
    daemon_mode: bool,

    #[structopt(short = "m", long = "home-assistant", about = "Broadcasts the current system to home assistant.")]
    home_assistant: bool,

    #[structopt(subcommand)]
    commands: Option<Cli>,
}

#[derive(Debug, StructOpt)]
enum Cli {
    Zone(ZoneOpts),
    Sys(SysOpts),
}

#[derive(StructOpt, Debug)]
struct ZoneOpts {
    id: u8,
    state: String,
}

#[derive(StructOpt, Debug)]
enum ZoneOptsArgs {
    On,
    Off,
    Status,
}

impl FromStr for ZoneOptsArgs {
    type Err = ();
    fn from_str(input: &str) -> Result<ZoneOptsArgs, Self::Err> {
        match input {
            "on" => Ok(ZoneOptsArgs::On),
            "off" => Ok(ZoneOptsArgs::Off),
            "status" => Ok(ZoneOptsArgs::Status),
            _ => {
                println!("Unrecognized subcommand.");
                exit(1);
            }
        }
    }
}


#[derive(StructOpt, Debug)]
enum SysOpts {
    /// Enables the system schedule
    On,
    /// Disables the system schedule
    Off,
    /// Runs the system
    Run,
    /// Runs the winterizing schedule
    Winterize,
    /// Prints the status of the system.
    Status,
    /// Tests the system.
    Test,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct MyConfig {
    sqlsprinkler_user: String,
    sqlsprinkler_pass: String,
    sqlsprinkler_host: String,
    sqlsprinkler_db: String,
    mqtt_host: String,
    mqtt_user: String,
    mqtt_pass: String,
    mqtt_enabled: bool,
    verbose: bool,
}

lazy_static! {
    static ref SETTINGS: RwLock<MyConfig> = RwLock::new(MyConfig::default());
}

const SETTINGS_FILE_PATH: &str = "/etc/sqlsprinkler/sqlsprinkler.conf";


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

/// Read the settings file from `/etc/sqlsprinlker/sqlsprinkler.conf` and load into memory.
fn read_settings() -> Result<(), confy::ConfyError> {
    let mut new_settings = SETTINGS.write().unwrap();
    *new_settings = confy::load_path(SETTINGS_FILE_PATH)?;
    Ok(())
}

fn get_settings() -> MyConfig {
    SETTINGS.read().unwrap().clone()
}

fn main() {
    let cli = Opts::from_args();
    println!("{:?}", cli);

    let daemon_mode = cli.daemon_mode;
    let version_mode = cli.version_mode;
    let home_assistant = cli.home_assistant;

    match read_settings() {
        Ok(..) => (),
        Err(e) => {
            println!("An error occurred while reading the config file: {}", e);
            exit(1)
        }
    };

    if version_mode {
        println!("SQLSprinkler v{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }

    if daemon_mode {
        turn_off_all_zones();
        daemon::run();
    }

    if home_assistant {
        mqtt::mqtt::mqtt_run();
    }

    if let Some(subcommand) = cli.commands {
        let zone_list = get_zones();
        match subcommand {
            // `sqlsprinkler zone ...`
            Cli::Zone(zone_state) => {
                let id = usize::from(zone_state.id);
                let _zone_list = zone_list;
                let my_zone: &Zone = match _zone_list.zones.get(id) {
                    Some(z) => z,
                    None => {
                        // Return the default zone.
                        panic!("Zone {} not found.", id);
                    }
                };
                match ZoneOptsArgs::from(zone_state.state.parse().unwrap()) {
                    ZoneOptsArgs::On => {
                        turn_off_all_zones();
                        my_zone.turn_on();
                    }
                    ZoneOptsArgs::Off => {
                        my_zone.turn_off();
                    }
                    ZoneOptsArgs::Status => {
                        let zone = &my_zone;
                        zone.get_zone_with_state().state;
                    }
                }
            }
            // `sqlsprinkler sys ...`
            Cli::Sys(sys_opts) => {
                match sys_opts {
                    SysOpts::On => {
                        if get_settings().verbose {
                            println!("Enabled system schedule");
                        }
                        set_system_status(true);
                    }
                    SysOpts::Off => {
                        if get_settings().verbose {
                            println!("Disabling system schedule.");
                        }
                        set_system_status(false);
                    }
                    SysOpts::Run => {
                        if get_system_status() {
                            if get_settings().verbose {
                                println!("Running the system schedule.");
                            }
                            sqlsprinkler::system::run();
                        } else {
                            if get_settings().verbose {
                                println!("System is not enabled, refusing.");
                            }
                        }
                    }
                    //TODO: Implement?!? Useful about once a year.
                    SysOpts::Winterize => {
                        if get_settings().verbose {
                            println!("Winterizing the system.");
                            winterize();
                        }
                    }
                    SysOpts::Status => {
                        let system_status = get_system_status();
                        let output = match system_status {
                            true => "enabled",
                            false => "disabled",
                        };
                        println!("The system is {}", output);
                    }
                    SysOpts::Test => {
                        turn_off_all_zones();
                        for zone in zone_list.zones {
                            zone.test_zone();
                        }
                    }
                }
            }
        }
    }
}