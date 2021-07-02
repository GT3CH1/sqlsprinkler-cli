use mysql::*;
// Copyright 2021 Gavin Pease
use std::env;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "how to use struct-opt crate")]
pub struct Opts {
    #[structopt(short = "v", parse(from_occurrences))]
    verbosity: u8,

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
struct Zone {
    name: String,
    gpio: i8,
    time: i8,
    enabled: bool,
    auto_off: bool,
    system_order: i8,
}
#[derive(Debug, PartialEq, Eq)]
struct SysStatus {
    status: bool,
}
fn main() {
    let cli = Opts::from_args();
    // Get the SQL database password, parse it.
    let key = "SQL_PASS";
    let mut pass = "".to_string();
    match env::var(key) {
        Ok(val) => pass = val,
        Err(e) => panic!("{}", e),
    }

    // Build the url for the connection
    let mut url = "mysql://sqlsprinkler:".to_owned();
    url.push_str(pass.as_str());
    url.push_str("@web.peasenet.com:3306/SQLSprinkler");

    let pool = mysql::Pool::new(url).unwrap();
    if let Some(subcommand) = cli.commands {
        match subcommand {
            // Parses the zone sub command, make sure that id is greater than 0.
            Cli::Zone(zone_state) => {
                let mut zone_toggle: bool = false;

                zone_toggle = zone_state.state == "on";
                if zone_state.id <= 0 { panic!("ID must be greater than 0"); }
                let zone_id = usize::from(zone_state.id) - 1;
                let zones = GetZones(pool);
                let my_zone = zones[zone_id].borrow();
                println!("Turning zone {} {:?}", zone_id, my_zone);
                //TODO: Make the GPIO pin turn on.
            }
            // Parses the zone sub command
            Cli::Sys(sys_opts) => {
                match sys_opts {
                    SysOpts::On => {
                        println!("Enabling system schedule.");
                        SetSystem(pool, true);
                    }
                    SysOpts::Off => {
                        println!("Disabling system schedule.");
                        SetSystem(pool, false);
                    }
                    SysOpts::Run => {
                        println!("Running the system schedule.")
                    }
                    SysOpts::Winterize => {
                        println!("Winterizing the system.");
                    }
                    SysOpts::Status => {
                        let status = match GetSystemStatus(pool) {
                            true => "enabled",
                            false => "disabled",
                        };
                        println!("The system is {}", status);
                    }
                }
            }
        }
    }
}

/// Gets a list of all the zones in this database
/// # Arguments
///     * `pool` The SQL connection pool to use to query for zones
/// # Returns
///     * `Vec<Zone>` A list of all the zones in the database.
fn GetZones(pool: Pool) -> Vec<Zone> {
    let all_zones: Vec<Zone> =
        pool.prep_exec("SELECT Name, Gpio, Runtime, Enabled, AutoOff, SystemOrder from Zones", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|row| {
                    let (name, gpio, time, enabled, auto_off, system_order) = mysql::from_row(row);
                    Zone {
                        name,
                        gpio,
                        time,
                        enabled,
                        auto_off,
                        system_order,
                    }
                }).collect()
            }).unwrap();
    return all_zones;
}

/// Enables or disables the system schedule
/// # Arguments
///     * `pool` The SQL connection pool used to toggle the system.
///     * `enabled` If true is passed in, the system is enabled. If false is used, the system is disabled.
fn SetSystem(pool: Pool, enabled: bool) {
    let query = format!("UPDATE Enabled set enabled = {}", enabled);
    pool.prep_exec(query, ()).unwrap();
}

/// Gets whether the system schedule is enabled or disabled
/// # Arguments
///     * `pool` The SQL connection pool used to toggle the system.
/// # Return
///     * `bool` True if the system is enabled, false if not.
fn GetSystemStatus(pool: Pool) -> bool {
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