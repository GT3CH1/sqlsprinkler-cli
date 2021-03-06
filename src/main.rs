// Copyright 2021 Gavin Pease

mod config;
mod mqtt;
mod sqlsprinkler;

use mysql::serde_json;
use std::fmt::Debug;
use std::process::exit;
use std::str::FromStr;

use structopt::StructOpt;

use crate::config::{get_settings, read_settings};
use crate::sqlsprinkler::system::{
    get_system_status, get_zones, set_system_status, turn_off_all_zones, winterize,
};
use crate::sqlsprinkler::zone::Zone;
use sqlsprinkler::daemon;

/// Holds the program's possible CLI options.
#[derive(Debug, StructOpt)]
#[structopt(name = "sqlsprinkler", about = "SQLSprinkler")]
pub struct Opts {
    /// Whether or not to print the version of the program
    #[structopt(short = "v", about = "Prints the version of SQLSprinkler.")]
    version_mode: bool,

    /// Whether or not to run in daemon mode
    #[structopt(
        short = "w",
        long = "daemon",
        about = "Launches the SQLSprinkler API web daemon."
    )]
    daemon_mode: bool,

    /// Whether or not to run in home assistant mode using MQTT
    #[structopt(
        short = "m",
        long = "home-assistant",
        about = "Broadcasts the current system to home assistant."
    )]
    home_assistant: bool,

    /// A list of sub commands to run
    #[structopt(subcommand)]
    commands: Option<Cli>,
}

/// The CLI subcommands.
#[derive(Debug, StructOpt)]
enum Cli {
    Zone(ZoneOpts),
    Sys(SysOpts),
}

/// Zone options
#[derive(StructOpt, Debug)]
struct ZoneOpts {
    id: u8,
    state: String,
}

/// The options for a zone.
/// Possible subcommands are:
/// - `on`: Turns the zone on.
/// - `off`: Turns the zone off.
/// - `status`: Returns the current status of the zone.
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

/// The system options. Possible subcommands are:
/// - `status`: Prints the current system status.
/// - `on`: Enables the system.
/// - `off`: Disables the system.
/// - `winterize`: Runs a winterization feature.
/// - `test`: Tests the system, so the user can check functionality.
/// - `run`: Runs the system as it is configured.
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

/// Main entry point for the SQLSprinkler program.
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
        mqtt::mqtt_client::mqtt_run();
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
                match zone_state.state.parse().unwrap() {
                    ZoneOptsArgs::On => {
                        turn_off_all_zones();
                        my_zone.turn_on();
                    }
                    ZoneOptsArgs::Off => {
                        my_zone.turn_off();
                    }
                    ZoneOptsArgs::Status => {
                        let zone = &my_zone;
                        println!(
                            "The zone is {}",
                            if zone.get_with_state().state {
                                "on"
                            } else {
                                "off"
                            }
                        );
                    }
                }
            }
            // `sqlsprinkler sys ...`
            Cli::Sys(sys_opts) => match sys_opts {
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
                    } else if get_settings().verbose {
                        println!("System is not enabled, refusing.");
                    }
                }
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
                        zone.test();
                    }
                }
            },
        }
    }
}
