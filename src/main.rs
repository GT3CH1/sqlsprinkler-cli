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
    id: i8,
    state: String,
}

#[derive(StructOpt, Debug)]
enum SysOpts {
    On,
    Off,
    Run,
    Winterize,
}

fn main() {
    let cli = Opts::from_args();
    if let Some(subcommand) = cli.commands {
        match subcommand {
            // Parses the "zone" sub command
            Cli::Zone(zone_state) => {
                let mut zone_toggle: bool = false;
                if zone_state.state == "on" {
                    zone_toggle = true;
                } else { zone_toggle = false; }
                println!("Turning system {} {}", zone_state.id, zone_toggle);
            }
            Cli::Sys(sys_opts) => {
                println!("{:?}",sys_opts);
            }
            _ => (),
        }
    }
}
