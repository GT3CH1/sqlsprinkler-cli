// Copyright 2021 Gavin Pease

mod daemon;
mod zone;

use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use std::env;
use crate::zone::{Zone};
use mysql::Pool;
use std::str::FromStr;
use std::process::exit;
use parking_lot::{RwLockReadGuard};
use std::sync::RwLock;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, StructOpt)]
#[structopt(name = "sqlsprinkler", about = "SQLSprinkler")]
pub struct Opts {
    #[structopt(short = "v", about = "Prints the version of SQLSprinkler.")]
    version_mode: bool,

    #[structopt(short = "w", long = "daemon", about = "Launches the SQLSprinkler API web daemon.")]
    daemon_mode: bool,

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
            _ => Err(()),
        }
    }
}


#[derive(StructOpt, Debug)]
enum SysOpts {
    On,
    Off,
    Run,
    Winterize,
    Status,
    Test,
}

#[derive(Debug, PartialEq, Eq)]
struct SysStatus {
    status: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct MyConfig {
    sqlsprinkler_user: String,
    sqlsprinkler_pass: String,
    sqlsprinkler_host: String,
    sqlsprinkler_db: String,
    verbose: bool,
}

lazy_static! {
    static ref SETTINGS: RwLock<MyConfig> = RwLock::new(MyConfig::default());
}

const SETTINGS_FILE_PATH: &str = "/etc/sqlsprinkler/sqlsprinkler.conf";

impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            sqlsprinkler_user: "".to_string(),
            sqlsprinkler_pass: "".to_string(),
            sqlsprinkler_host: "".to_string(),
            sqlsprinkler_db: "".to_string(),
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
    let daemon_mode = cli.daemon_mode;
    let version_mode = cli.version_mode;
    read_settings();
    if version_mode {
        println!("SQLSprinkler v{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }
    turn_off_all_zones();
    if daemon_mode {
        daemon::run();
    }
    let zone_list = zone::get_zones();
    if let Some(subcommand) = cli.commands {
        match subcommand {
            // Parses the zone sub command, make sure that id is greater than 0.
            Cli::Zone(zone_state) => {
                let id = usize::from(zone_state.id);
                let _zone_list = zone_list;
                let my_zone: Zone = _zone_list.zones.get(id).unwrap().clone();
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
                        zone.is_on();
                    }
                }
            }
            // Parses the zone sub command
            Cli::Sys(sys_opts) => {
                match sys_opts {
                    SysOpts::On => {
                        if get_settings().verbose {
                            println!("Enabled system schedule");
                        }
                        set_system(true);
                    }
                    SysOpts::Off => {
                        if get_settings().verbose {
                            println!("Disabling system schedule.");
                        }
                        set_system(false);
                    }
                    SysOpts::Run => {
                        if get_system_status() {
                            if get_settings().verbose {
                                println!("Running the system schedule.");
                            }
                            run_system();
                        } else {
                            if get_settings().verbose {
                                println!("System is not enabled, refusing.");
                            }
                        }
                    }
                    SysOpts::Winterize => {
                        if get_settings().verbose {
                            println!("Winterizing the system.");
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

/// Gets a connection to a MySQL database
/// # Return
///     `Pool` A connection to the SQL database.
fn get_pool() -> Pool {
    // Build the url for the connection
    let reader = get_settings();
    let url = format!("mysql://{}:{}@{}:3306/{}",
                      reader.sqlsprinkler_user, reader.sqlsprinkler_pass, reader.sqlsprinkler_host, reader.sqlsprinkler_db);

    let pool = match mysql::Pool::new(url) {
        Ok(p) => p,
        Err(_e) => {
            println!("Could not connect! Did you set the username/password correctly?");
            exit(1);
        }
    };
    pool
}

/// Enables or disables the system schedule
/// # Arguments
///     * `pool` The SQL connection pool used to toggle the system.
///     * `enabled` If true is passed in, the system is enabled. If false is used, the system is disabled.
fn set_system(enabled: bool) {
    let pool = get_pool();
    let query = format!("UPDATE Enabled set enabled = {}", enabled);
    pool.prep_exec(query, ()).unwrap();
}

/// Gets whether the system schedule is enabled or disabled
/// # Arguments
///     * `pool` The SQL connection pool used to toggle the system.
/// # Return
///     * `bool` True if the system is enabled, false if not.
fn get_system_status() -> bool {
    let pool = get_pool();
    let query = format!("SELECT enabled FROM Enabled");
    let sys_status: Vec<SysStatus> =
        pool.prep_exec(query, ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|row| {
                    let status = mysql::from_row(row);
                    SysStatus {
                        status
                    }
                }).collect()
            }).unwrap();
    return sys_status[0].status;
}

/// Turns off all the zones in the system
fn turn_off_all_zones() {
    let zone_list = zone::get_zones();
    for zone_in_list in &zone_list.zones {
        zone_in_list.turn_off();
    }
}

/// Runs the system based on the schedule
fn run_system() {
    let zone_list = zone::get_zones();
    for zone in &zone_list.zones {
        if zone.enabled {
            zone.run_zone();
        }
    }
}