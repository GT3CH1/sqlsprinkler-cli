// Copyright 2021 Gavin Pease

mod daemon;
mod zone;

use structopt::StructOpt;
use rppal::gpio::Gpio;
use std::{error::Error, env, thread, time};
use crate::zone::{Zone};
use mysql::Pool;
use std::str::FromStr;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "how to use struct-opt crate")]
pub struct Opts {
    #[structopt(short = "v", parse(from_occurrences))]
    verbosity: u8,

    /// Launches the SQLSprinkler API web daemon.
    #[structopt(short = "w", long = "daemon", about = "Launches the SQLSprinkler API web daemon")]
    daemon_mode: bool,

    /// Output everything in JSON.
    #[structopt(long = "json")]
    json_output: bool,

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
}

impl FromStr for ZoneOptsArgs {
    type Err = ();
    fn from_str(input: &str) -> Result<ZoneOptsArgs, Self::Err> {
        match input {
            "on" => Ok(ZoneOptsArgs::On),
            "off" => Ok(ZoneOptsArgs::Off),
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
    let json_output = cli.json_output;
    if daemon_mode {
        daemon::run();
    }
    let zoneList = zone::get_zones();
    if let Some(subcommand) = cli.commands {
        match subcommand {
            // Parses the zone sub command, make sure that id is greater than 0.
            Cli::Zone(zone_state) => {
                let mut zone_toggle: bool = false;
                if zone_state.id < 0 { panic!("ID must be greater or equal to 0"); }
                let id = usize::from(zone_state.id);
                let _zoneList = zoneList;
                let my_zone: Zone = _zoneList.zones.get(id).unwrap().clone();
                match ZoneOptsArgs::from(zone_state.state.parse().unwrap()) {
                    ZoneOptsArgs::On => {
                        zone::set_pin_zone(my_zone, true);
                    }
                    ZoneOptsArgs::Off => {
                        zone::set_pin_zone(my_zone, false);
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
                        if !json_output {
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

/// Turns the given pin on or off
/// # Params
///     * `pin` The pin we want to control
///     * `state` True if we want the pin to turn on, false otherwise.
fn set_pin(pin: u8, state: bool) -> Result<(), Box<dyn Error>> {
    let mut gpio = Gpio::new()?.get(pin).unwrap().into_output();
    if state {
        gpio.set_low();
    } else {
        gpio.set_high();
    }
    Ok(())
}

/// Turns off all the pins in the system
fn turn_off_all_pins() {
    let zone_list = zone::get_zones();
    for zone_in_list in &zone_list.zones {
        let my_zone = zone_in_list;
        set_pin(my_zone.gpio as u8, false);
    }
}

/// Get the gpio state
/// # Params
///      * `zone` The zone we want to get the state of.
/// # Return
///     * `bool` Whether or not the pin is set to low.
fn get_pin_state(pin: u8) -> bool {
    let gpio = Gpio::new().unwrap();
    let gpio = gpio.get(pin).unwrap().into_output();
    return gpio.is_set_low();
}

/// Runs the system based on the schedule
fn run_system() {
    let zone_list = zone::get_zones();
    println!("Running system");
    for zone in &zone_list.zones {
        if zone.enabled {
            println!("Running zone {}",zone.name);
            set_pin(zone.gpio as u8, true);
            let run_time = time::Duration::from_secs((zone.time * 60) as u64);
            thread::sleep(run_time);
            set_pin(zone.gpio as u8, false);
        }
    }
}