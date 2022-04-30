// Copyright 2021 Gavin Pease

mod sqlsprinkler;

use std::{env, thread};
use std::collections::HashMap;
use std::fmt::Debug;
use std::process::exit;
use std::str::FromStr;
use std::sync::RwLock;
use std::time::Duration;
use mysql::serde_json;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use paho_mqtt as mqtt;

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

    #[structopt(short = "ha", long = "home-assistant", about = "Broadcasts the current system to home assistant.")]
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
    static ref ZONES: RwLock<HashMap<String, Zone>> = RwLock::new(HashMap::new());
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

        let mqtt_host = get_settings().mqtt_host;
        let mqtt_user = get_settings().mqtt_user;
        let mqtt_pass = get_settings().mqtt_pass;

        // create the mqtt client
        let mqtt_client = mqtt::AsyncClient::new(mqtt_host.to_string()).unwrap();
        let opts = mqtt::ConnectOptionsBuilder::new()
            .user_name(mqtt_user)
            .password(mqtt_pass)
            .finalize();

        mqtt_client.connect_with_callbacks(opts, on_connect_success, on_connect_failure);

        // create a hashset of topics and zone

        // Set a closure to be called whenever the client connection is established.
        mqtt_client.set_connected_callback(|cli: &mqtt::AsyncClient| {
            println!("Connected.");
        });

        // Set a closure to be called whenever the client loses the connection.
        // It will attempt to reconnect, and set up function callbacks to keep
        // retrying until the connection is re-established.
        mqtt_client.set_connection_lost_callback(|cli: &mqtt::AsyncClient| {
            println!("Connection lost. Attempting reconnect.");
            thread::sleep(Duration::from_millis(2500));
            cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
        });

        // Start listening for mqtt messages
        mqtt_client.set_message_callback(|_cli, msg| {
            if let Some(msg) = msg {
                let topic = msg.topic();
                let payload_str = msg.payload_str();
                println!("{} - {}", topic, payload_str);
                // Iterate through the zones and turn on the zones that match the topic
                for zone in ZONES.read().unwrap().iter() {
                    if topic == zone.0 {
                        // check if the payload matches sqlsprinkler_zone_<zone_id>
                        if topic == format!("sqlsprinkler_zone_{}/command", zone.1.id) {
                            turn_off_all_zones();
                            if payload_str.clone() == "ON" {
                                zone.1.turn_on();
                            }
                        }
                        // check if the payload matches sqlsprinkler_zone_<zone_id>_time
                        if topic == format!("sqlsprinkler_zone_{}_time/command", zone.1.id) {
                            let time = payload_str.parse::<u64>().unwrap();
                            let mut new_zone = zone.1.clone();
                            new_zone.time = time;
                            zone.1.update_zone(new_zone);

                        }
                        // check if payload matches sqlsprinkler_zone_<zone_id>_auto_off_state
                        if topic == format!("sqlsprinkler_zone_{}_auto_off_state/command", zone.1.id) {
                            let auto_off_state = payload_str.parse::<bool>().unwrap();
                            let mut new_zone = zone.1.clone();
                            new_zone.auto_off = auto_off_state;
                            zone.1.update_zone(new_zone);
                        }
                        // check if payload matches sqlsprinkler_zone_<zone_id>_enabled_state
                        if topic == format!("sqlsprinkler_zone_{}_enabled_state/command", zone.1.id) {
                            let enabled_state = payload_str.parse::<bool>().unwrap();
                            let mut new_zone = zone.1.clone();
                            new_zone.enabled = enabled_state;
                            zone.1.update_zone(new_zone);
                        }
                    }
                }
                // Check if topic is sqlsprinkler_system/command
                if topic == "sqlsprinkler_system/command" {
                    if payload_str.clone() == "ON" {
                        set_system_status(true)
                    } else if payload_str.clone() == "OFF" {
                        set_system_status(false)
                    }
                }
            }
        });

        loop {
            thread::sleep(Duration::from_millis(5000));
            // Send current status of all zones
            for zone in get_zones().zones {
                // send current status
                let mut topic = format!("sqlsprinkler_zone_{}/status", zone.id);
                let mut payload = format!("{}",  if zone.get_zone_with_state().state { "ON" } else { "OFF" });
                let mut msg = mqtt::Message::new(topic, payload, 0);
                mqtt_client.publish(msg);

                // send current time
                topic = format!("sqlsprinkler_zone_{}_time/status", zone.id);
                payload = format!("{}", zone.time);
                msg = mqtt::Message::new(topic, payload, 0);
                mqtt_client.publish(msg);

                // send current auto off state
                topic = format!("sqlsprinkler_zone_{}_auto_off_state/status", zone.id);
                payload = format!("{}", if zone.auto_off { "ON" } else { "OFF" });
                msg = mqtt::Message::new(topic, payload, 0);
                mqtt_client.publish(msg);

                // send current enabled state
                topic = format!("sqlsprinkler_zone_{}_enabled_state/status", zone.id);
                payload = format!("{}", if zone.enabled { "ON" } else { "OFF" });
                msg = mqtt::Message::new(topic, payload, 0);
                mqtt_client.publish(msg);
            }
            // send current status of the system switch
            let topic = format!("sqlsprinkler_system/status");
            let payload = format!("{}", if get_system_status() { "ON" } else { "OFF" });
            let msg = mqtt::Message::new(topic, payload, 0);
            mqtt_client.publish(msg);
        }

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

fn on_connect_success(cli: &mqtt::AsyncClient, _msgid: u16) {
    println!("Connection succeeded");

    //broadcast all of the zones.
    for zone in get_zones().zones {
        // Broadcast the zone discovery message to the MQTT broker (as a switch)
        let mut zone_topic = format!("homeassistant/switch/sqlsprinkler_zone_{}/config", &zone.id);
        let mut mqtt_sprinkler = mqttsprinkler::MqttSprinkler::sprinkler(&zone);
        let mut payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
        let mut msg = mqtt::Message::new(zone_topic.clone(), payload.clone(), 0);
        println!("Sending MQTT message: {}", payload.clone());
        cli.publish(msg);
        ZONES.write().unwrap().insert(mqtt_sprinkler.cmd_t, zone.clone());

        // Broadcast the zone time to the mqtt broker (as a number)
        zone_topic = format!("homeassistant/number/sqlsprinkler_zone_{}_time/config", &zone.id);
        mqtt_sprinkler = mqttsprinkler::MqttSprinkler::zone_time(&zone);
        payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
        msg = mqtt::Message::new(zone_topic.clone(), payload.clone(), 0);
        println!("Sending MQTT message: {}", payload.clone());
        cli.publish(msg);
        ZONES.write().unwrap().insert(mqtt_sprinkler.cmd_t, zone.clone());

        // Broadcast the zone auto off state to the mqtt broker (as a switch)
        zone_topic = format!("homeassistant/switch/sqlsprinkler_zone_{}_auto_off/config", &zone.id);
        mqtt_sprinkler = mqttsprinkler::MqttSprinkler::zone_auto_off(&zone);
        payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
        msg = mqtt::Message::new(zone_topic.clone(), payload.clone(), 0);
        println!("Sending MQTT message: {}", payload.clone());
        cli.publish(msg);
        ZONES.write().unwrap().insert(mqtt_sprinkler.cmd_t, zone.clone());

        // Broadcast the zone enabled state to the mqtt broker (as a switch)
        zone_topic = format!("homeassistant/switch/sqlsprinkler_zone_{}_enabled/config", &zone.id);
        mqtt_sprinkler = mqttsprinkler::MqttSprinkler::zone_enabled(&zone);
        payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
        msg = mqtt::Message::new(zone_topic.clone(), payload.clone(), 0);
        println!("Sending MQTT message: {}", payload.clone());
        cli.publish(msg);
        ZONES.write().unwrap().insert(mqtt_sprinkler.cmd_t, zone.clone());
    }
    // broadcast the system toggle
    let topic = format!("homeassistant/switch/sqlsprinkler_system/config");
    let system = mqttsprinkler::MqttSprinkler::system();
    let payload = serde_json::to_string(&system).unwrap();
    let msg = mqtt::Message::new(topic, payload.clone(), 0);
    ZONES.write().unwrap().insert(system.cmd_t, Zone::default());
    println!("Sending MQTT message: {}", payload.clone());
    cli.publish(msg);
    // Subscrive to all zone topics
    for zone in ZONES.read().unwrap().keys() {
        cli.subscribe(zone, 0);
        println!("Subscribed to {}", zone);
    }

}

// Callback for a failed attempt to connect to the server.
// We simply sleep and then try again.
//
// Note that normally we don't want to do a blocking operation or sleep
// from  within a callback. But in th`is case, we know that the client is
// *not* conected, and thus not doing anything important. So we don't worry
// too much about stopping its callback thread.
fn on_connect_failure(cli: &mqtt::AsyncClient, _msgid: u16, rc: i32) {
    println!("Connection attempt failed with error code {}.\n", rc);
    thread::sleep(Duration::from_millis(2500));
    cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
}