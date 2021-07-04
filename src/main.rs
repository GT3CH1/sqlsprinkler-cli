// Copyright 2021 Gavin Pease

mod daemon;
mod zone;

use structopt::StructOpt;
use std::borrow::Borrow;
use mysql::Pool;
use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;
use std::env;
use std::error::Error;

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
    /// The ID of the zone.
    id: u8,
    state: String,
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
    let pool = get_pool();
    if let Some(subcommand) = cli.commands {
        match subcommand {
            // Parses the zone sub command, make sure that id is greater than 0.
            Cli::Zone(zone_state) => {
                let mut zone_toggle: bool = false;

                zone_toggle = zone_state.state == "on";
                if zone_state.id < 0 { panic!("ID must be greater or equal to 0"); }
                let zone_id = usize::from(zone_state.id) ;
                let zones = get_zones(pool);
                let my_zone = zones[zone_id].borrow();
                println!("Turning zone {} {:?}", zone_id, my_zone);
                set_pin(my_zone.gpio as u8, zone_toggle);
            }
            // Parses the zone sub command
            Cli::Sys(sys_opts) => {
                match sys_opts {
                    SysOpts::On => {
                        set_system(pool, true);
                    }
                    SysOpts::Off => {
                        println!("Disabling system schedule.");
                        set_system(pool, false);
                    }
                    SysOpts::Run => {
                        println!("Running the system schedule.")
                    }
                    SysOpts::Winterize => {
                        println!("Winterizing the system.");
                    }
                    SysOpts::Status => {
                        let system_status = get_system_status(pool);
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

fn get_pool() -> Pool {
    // Get the SQL database password, parse it.
    let key = "SQL_PASS";
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
    let mut url = format!("mysql://{}:{}@{}:3306/SQLSprinkler",user.as_str(),pass.as_str(),host.as_str());

    let pool = mysql::Pool::new(url).unwrap();
    return pool;
}

/// Gets a list of all the zones in this database
/// # Arguments
///     * `pool` The SQL connection pool to use to query for zones
/// # Returns
///     * `Vec<Zone>` A list of all the zones in the database.
fn get_zones(pool: Pool) -> Vec<zone::Zone> {
    let all_zones: Vec<zone::Zone> =
        pool.prep_exec("SELECT Name, Gpio, Time, Enabled, AutoOff, SystemOrder, ID from Zones ORDER BY SystemOrder", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|row| {
                    let (name, gpio, time, enabled, auto_off, system_order, id) = mysql::from_row(row);
                    zone::Zone {
                        name,
                        gpio,
                        time,
                        enabled,
                        auto_off,
                        system_order,
                        id,
                    }
                }).collect()
            }).unwrap();
    return all_zones;
}

/// Enables or disables the system schedule
/// # Arguments
///     * `pool` The SQL connection pool used to toggle the system.
///     * `enabled` If true is passed in, the system is enabled. If false is used, the system is disabled.
fn set_system(pool: Pool, enabled: bool) {
    let query = format!("UPDATE Enabled set enabled = {}", enabled);
    pool.prep_exec(query, ()).unwrap();
}

/// Gets whether the system schedule is enabled or disabled
/// # Arguments
///     * `pool` The SQL connection pool used to toggle the system.
/// # Return
///     * `bool` True if the system is enabled, false if not.
fn get_system_status(pool: Pool) -> bool {
    let query = format!("SELECT enabled FROM Enabled");
    let sys_status: Vec<SysStatus> =
        pool.prep_exec(query, ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|row| {
                    let (status) = mysql::from_row(row);
                    SysStatus {
                        status
                    }
                }).collect()
            }).unwrap();
    return sys_status[0].status;
}

/// Adds a new zone
/// # Params
///     * `_zone` The new zone we want to add.
fn add_new_zone(_zone: zone::ZoneAdd) {
    let pool = get_pool();
    let query = format!("INSERT into `Zones` (`Name`, `Gpio`, `Time`, `AutoOff`, `Enabled`) VALUES \
     ( '{}','{}','{}',{},{} )", _zone.name, _zone.gpio, _zone.time, _zone.auto_off, _zone.enabled);
    pool.prep_exec(query, ());
}

/// Deletes the given zone
/// # Params
///     * `_zone` The zone we are deleting
fn delete_zone(_zone: zone::ZoneDelete) {
    let pool = get_pool();
    let query = format!("DELETE FROM `Zones` WHERE id = {}", _zone.id);
    pool.prep_exec(query, ());
}

/// Updates a zone with the given id to the values contained in this new zone.
/// # Params
///     * `_zone` The zone containing the same id, but new information.
fn update_zone(_zone: zone::Zone) {
    let pool = get_pool();
    let query = format!("UPDATE Zones SET Name='{}', Gpio={}, Time={},AutoOff={},Enabled={},SystemOrder={} WHERE ID={}"
                        , _zone.name, _zone.gpio, _zone.time, _zone.auto_off, _zone.enabled, _zone.system_order, _zone.id);
    println!("{}",query);
    pool.prep_exec(query, ());
}

/// Updates the system order of the given zone to the given order, and then updates it in the database
/// # Params
///     * `order` The number representing the order of the zone
///     * `zone` The zone we want to change the order of.
fn change_zone_ordering(order: i8, zone: zone::Zone) {
   let new_zone_order = zone::Zone {
       name: zone.name,
       gpio: zone.gpio,
       time: zone.time,
       enabled: zone.enabled,
       auto_off: zone.auto_off,
       system_order: order,
       id: zone.id
   };
    update_zone(new_zone_order);
}

/// Turns the given pin on or off
/// # Params
///     * `pin` The pin we want to control
///     * `state` True if we want the pin to turn on, false otherwise.
fn set_pin(pin: u8, state: bool) -> Result<(), Box<dyn Error>>{
    let mut gpio = Gpio::new()?.get(pin).unwrap().into_output();
    if state {
        gpio.set_low();
    } else {
        gpio.set_high();
    }
    Ok(())
}

/// Sets the given zones gpio to the state we want
/// # Params
///     * `zone` The zone we want to control
///     * `state` The state we want the pin to be at - true for on, false for off.
fn set_pin_zone(zone: zone::Zone, state: bool){
    // Ensure all the pins are turned off.
    set_pin(zone.gpio as u8, state);
}

/// Turns off all the pins in the system
fn turn_off_all_pins() {
    let zone_list = get_zones(get_pool());
    for zone_in_list in &zone_list {
        let zone = zone::Zone::from(zone_in_list);
        set_pin(zone.gpio as u8,false);
    }
}

/// Get the gpio state
/// # Params
///      * `zone` The zone we want to get the state of.
/// # Return
///     * `bool` Whether or not the pin is set to low.
fn get_pin_state(pin: u8) -> bool {
    let gpio = Gpio::new().unwrap();
    let mut gpio = gpio.get(pin).unwrap().into_output();
    return gpio.is_set_low();
}