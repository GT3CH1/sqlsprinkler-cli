// Copyright 2021 Gavin Pease

mod daemon;
mod zone;

use structopt::StructOpt;
use std::{env, thread};
use crate::zone::{Zone};
use mysql::Pool;
use std::str::FromStr;

#[derive(Debug, StructOpt)]
#[structopt(name = "sqlsprinkler", about = "SQLSprinkler")]
pub struct Opts {
    #[structopt(short = "v", parse(from_occurrences))]
    verbosity: u8,

    /// Launches the SQLSprinkler API web daemon.
    #[structopt(short = "w", long = "daemon", about = "Launches the SQLSprinkler API web daemon")]
    daemon_mode: bool,

    // SUBCOMMANDS
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
}

#[derive(Debug, PartialEq, Eq)]
struct SysStatus {
    status: bool,
}

fn main() {
    let cli = Opts::from_args();
    let daemon_mode = cli.daemon_mode;
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
                        println!("Enable system schedule");
                        set_system(true);
                    }
                    SysOpts::Off => {
                        println!("Disabling system schedule.");
                        set_system(false);
                    }
                    SysOpts::Run => {
                        if get_system_status() {
                            println!("Running the system schedule.");
                            run_system();
                        } else {
                            println!("System is not enabled, refusing.");
                        }
                    }
                    SysOpts::Winterize => {
                        println!("Winterizing the system.");
                    }
                    SysOpts::Status => {
                        let system_status = get_system_status();
                        let output = match system_status {
                            true => "enabled",
                            false => "disabled",
                        };
                        println!("The system is {}", output);
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
    // Get the SQL database password, parse it.
    let mut user = "".to_string();
    let mut pass = "".to_string();
    let mut host = "".to_string();
    match env::var("SQL_PASS") {
        Ok(val) => pass = val,
        Err(e) => println!("{}", e),
    }
    match env::var("SQL_HOST") {
        Ok(val) => host = val,
        Err(e) => println!("{}", e),
    }
    match env::var("SQL_USER") {
        Ok(val) => user = val,
        Err(e) => println!("{}", e),
    }
    // Build the url for the connection
    let url = format!("mysql://{}:{}@{}:3306/SQLSprinkler", user.as_str(), pass.as_str(), host.as_str());

    let pool = mysql::Pool::new(url).unwrap();
    return pool;
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
    println!("Running system");
        for zone in &zone_list.zones {
            if zone.enabled {
                zone.run_zone();
            }
        }
}