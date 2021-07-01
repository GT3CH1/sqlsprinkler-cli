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

fn main() {
    let cli = Opts::from_args();
    let key = "SQL_PASS";
    let mut pass = "".to_string();
    match env::var(key) {
        Ok(val) => pass = val,
        Err(e) => println!("{}", e)
    }
    let mut url = "mysql://sqlsprinkler:".to_owned();
    url.push_str(pass.as_str());
    url.push_str("@web.peasenet.com:3306/SQLSprinkler");

    let pool = mysql::Pool::new(url).unwrap();
    if let Some(subcommand) = cli.commands {
        match subcommand {
            // Parses the "zone" sub command
            Cli::Zone(zone_state) => {
                let mut zone_toggle: bool = false;
                if zone_state.state == "on" {
                    zone_toggle = true;
                } else { zone_toggle = false; }
                let zones = GetZones(pool);
                println!("Turning system {} {}", zones[usize::from(zone_state.id) - 1].name, zone_toggle);
            }
            Cli::Sys(sys_opts) => {
                println!("{:?}", sys_opts);
            }
            _ => (),
        }
    }
}

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