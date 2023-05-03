// Copyright 2021 Gavin Pease

extern crate core;

mod config;
mod mqtt;
mod sqlsprinkler;

use crate::config::{get_settings, read_settings};
use crate::sqlsprinkler::system::{
    get_system_status, get_zones, set_system_status, turn_off_all_zones, winterize,
};
use crate::sqlsprinkler::zone::{Zone, ZoneAdd};
use chrono::Local;
use env_logger::fmt::{Color, Formatter};
use env_logger::{Builder, Env};
use log::{error, info, warn, Level, Record};
use sqlsprinkler::daemon;
use std::fmt::Debug;
use std::io::Write;
use std::process::exit;
use std::str::FromStr;
use structopt::StructOpt;
use crate::sqlsprinkler::{get_pool, zone};

/// Holds the program's possible CLI options.
#[derive(Debug, StructOpt)]
#[structopt(name = "sqlsprinkler", about = "SQLSprinkler")]
pub struct Opts {
    /// Whether or not to print the version of the program
    #[structopt(short = "V", about = "Prints the version of SQLSprinkler.")]
    version_mode: bool,

    /// Whether or not to use verbose output
    #[structopt(short = "v", long = "verbose", about = "Verbose mode")]
    verbose_mode: bool,

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
enum ZoneOpts {
    State(ZoneState),
    Add(ZoneAdd),
    Delete(ZoneDelete),
    Modify(ZoneModify),
    List,
}

#[derive(StructOpt, Debug)]
struct ZoneModify {
    id: u8,
    name: String,
    gpio: u8,
    time: u64,
    #[structopt(parse(try_from_str))]
    enabled: bool,
    #[structopt(parse(try_from_str))]
    auto_off: bool,
    order: u8,
}

#[derive(StructOpt, Debug)]
struct ZoneDelete {
    id: u8,
}

#[derive(StructOpt, Debug)]
struct ZoneState {
    /// The ID of the zone to modify.
    id: u8,
    /// The state of the zone.
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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let cli = Opts::from_args();

    let daemon_mode = cli.daemon_mode;
    let version_mode = cli.version_mode;
    let home_assistant = cli.home_assistant;
    let verbose_mode = cli.verbose_mode;

    match read_settings() {
        Ok(..) => (),
        Err(e) => {
            error!("An error occurred while reading the config file: {}", e);
            exit(1)
        }
    };
    let mut log_level = "info";

    if !verbose_mode && !get_settings().verbose {
        log_level = "warn";
    }

    Builder::from_env(Env::default().default_filter_or(log_level))
        .format(|buf, record| log_formatter(buf, record))
        .init();

    if version_mode {
        info!("SQLSprinkler v{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }

    if daemon_mode {
        info!("Starting SQLSprinkler daemon...");
        match turn_off_all_zones().await {
            Ok(..) => (),
            Err(e) => {
                error!("An error occurred while turning off all zones: {}", e);
                // exit(1);
            }
        }
        std::thread::spawn(move || async move {
            daemon::run().await;
        });
    }

    if home_assistant {
        info!("Starting home assistant/mqtt integration...");
        match mqtt::mqtt_client::mqtt_run().await {
            Ok(..) => (),
            Err(e) => {
                error!("An error occurred while running the mqtt client: {}", e);
                exit(1);
            }
        }
    }

    if let Some(subcommand) = cli.commands {
        let zone_list = get_zones().await?;
        match subcommand {
            // `sqlsprinkler zone ...`
            Cli::Zone(zone_state) => {
                match zone_state {
                    ZoneOpts::State(x) => {
                        let id = usize::from(x.id);
                        let _zone_list = zone_list;
                        // find the zone with the matching id.
                        let my_zone = match _zone_list.zones.into_iter().find(|z| z.id == (id as i8)) {
                            None => {
                                error!("Unable to find zone with id {}", id);
                                exit(1);
                            }
                            Some(z) => z
                        };
                        match x.state.parse().unwrap() {
                            ZoneOptsArgs::On => {
                                match turn_off_all_zones().await {
                                    Ok(_) => (),
                                    Err(e) => {
                                        error!("An error occurred while turning off all zones: {}", e);
                                        exit(1);
                                    }
                                }
                                my_zone.turn_on();
                            }
                            ZoneOptsArgs::Off => {
                                my_zone.turn_off();
                            }
                            ZoneOptsArgs::Status => {
                                let state = if my_zone.get_with_state().state {
                                    "on"
                                } else {
                                    "off"
                                };
                                info!("Zone {} ({}) is turned {}.", my_zone.id, my_zone.Name, state)
                            }
                        }
                    }
                    ZoneOpts::Add(x) => {
                        zone::add(x).await?;
                    }
                    ZoneOpts::Delete(x) => {
                        let query = sqlx::query!("DELETE FROM Zones WHERE id = ?", x.id)
                            .execute(&get_pool().await?)
                            .await;
                        match query {
                            Ok(_) => info!("Zone deleted successfully."),
                            Err(e) => error!("An error occurred while deleting the zone: {}", e),
                        }
                    }
                    ZoneOpts::Modify(x) => {
                        let query = sqlx::query!("UPDATE Zones SET name=?, gpio=?, time=?, enabled=?, autooff=?, systemorder=? WHERE id = ?", x.name, x.gpio, x.time, x.enabled, x.auto_off, x.order, x.id)
                            .execute(&get_pool().await?)
                            .await;
                        match query {
                            Ok(_) => info!("Zone modified successfully."),
                            Err(e) => error!("An error occurred while modifying the zone: {}", e),
                        }
                    }
                    ZoneOpts::List => {
                        // fetch all zones and print them
                        let list = get_zones().await?;
                        for zone in list.zones {
                            println!("{}", zone);
                        }
                    }
                }
            }
            // `sqlsprinkler sys ...`
            Cli::Sys(sys_opts) => match sys_opts {
                SysOpts::On => {
                    match set_system_status(true).await {
                        Ok(..) => {
                            info!("System schedule enabled successfully.");
                        }
                        Err(e) => {
                            error!("An error occurred while enabling the system schedule: {}", e);
                            exit(1);
                        }
                    };
                }
                SysOpts::Off => {
                    info!("Disabling system schedule.");
                    match set_system_status(false).await {
                        Ok(..) => {
                            info!("System schedule disabled successfully.");
                        }
                        Err(e) => {
                            error!("An error occurred while disabling the system schedule: {}", e);
                            exit(1);
                        }
                    }
                }
                SysOpts::Run => {
                    if get_system_status().await? {
                        info!("Running the system schedule.");
                        match sqlsprinkler::system::run().await {
                            Ok(..) => {
                                info!("System schedule ran successfully.");
                            }
                            Err(e) => {
                                error!("An error occurred while running the system schedule: {}", e);
                                exit(1);
                            }
                        };
                    } else {
                        warn!("System is not enabled, refusing.");
                    }
                }
                SysOpts::Winterize => {
                    info!("Winterizing the system.");
                    match winterize().await {
                        Ok(..) => {
                            info!("System winterized successfully.");
                        }
                        Err(e) => {
                            error!("An error occurred while winterizing the system: {}", e);
                            exit(1);
                        }
                    }
                }
                SysOpts::Status => {
                    let system_status = match get_system_status().await {
                        Ok(status) => status,
                        Err(e) => {
                            error!("An error occurred while getting the system status: {}", e);
                            exit(1);
                        }
                    };
                    let output = match system_status {
                        true => "enabled",
                        false => "disabled",
                    };
                    info!("The system is {}", output);
                }
                SysOpts::Test => {
                    match turn_off_all_zones().await {
                        Ok(_) => (),
                        Err(e) => {
                            error!("An error occurred while turning off all zones: {}", e);
                            exit(1);
                        }
                    }
                    for zone in zone_list.zones {
                        zone.test();
                    }
                }
            },
        }
    }
    Ok(())
}

fn log_formatter(buf: &mut Formatter, record: &Record) -> Result<(), std::io::Error> {
    let mut style = buf.style();
    let mut time_style = buf.style();
    time_style.set_color(Color::Rgb(0, 238, 255));
    match record.level() {
        Level::Error => {
            style.set_color(Color::Red);
            style.set_bold(true)
        }
        Level::Warn => {
            style.set_color(Color::Yellow);
            style.set_bold(true)
        }
        Level::Info => style.set_color(Color::White),
        Level::Debug => style.set_color(Color::Cyan),
        Level::Trace => style.set_color(Color::Magenta),
    };
    writeln!(
        buf,
        "{} [{}] - {}",
        time_style.value(Local::now().format("%m-%d-%Y %H:%M:%S")),
        style.value(record.level()),
        style.value(record.args())
    )
}
