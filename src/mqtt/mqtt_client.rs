use crate::mqtt::mqttdevice;
use crate::{
    get_settings, get_system_status, get_zones, serde_json, set_system_status, turn_off_all_zones,
    Zone,
};
use lazy_static::lazy_static;
use paho_mqtt as mqtt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

lazy_static! {
    // create a hashmap of topics and zone
    static ref ZONES: RwLock<HashMap<String, Zone>> = RwLock::new(HashMap::new());
}

/// Runs the MQTT client for the sprinkler system.
/// Connects to the MQTT broker given in the settings, and authenticates with the username and password.
/// Then, it will publish a auto discovery message for home assistant, followed by the current state of the system.
/// Finally, it will subscribe to the following topics:
///    - `sqlsprinkelr_zone_<zone_id>/command`: will subscribe to the topic for turning on/off a zone.
///    - `sqlsprinkler_zone_<zone_id>_timer/command`: will subscribe to the topic for setting the timer for a zone.
///    - `sqlsprinkelr_zone_<zone_id>_auto_off_state/command`: will subscribe to the topic for setting the auto off state for a zone.
///    - `sqlsprinkler_zone_<zone_id>_enabled/command`: will subscribe to the topic for setting the enabled state for a zone.
///    - `sqlsprinkler_system/command`: will subscribe to the topic for turning on/off the entire system.
/// The following topics will be published:
///    - `sqlsprinkler_system/state`: will publish the current state of the system.
///    - `sqlsprinkler_zone_<zone_id>/state`: will publish the current state of a zone.
///    - `sqlsprinkler_zone_<zone_id>_timer/state`: will publish the current timer for a zone.
///    - `sqlsprinkliner_zone_<zone_id>_auto_off_state/state`: will publish the current auto off state for a zone.
///    - `sqlsprinkler_zone_<zone_id>_enabled/state`: will publish the current enabled state for a zone.
pub fn mqtt_run() -> ! {
    let mqtt_host = get_settings().mqtt_host;
    let mqtt_user = get_settings().mqtt_user;
    let mqtt_pass = get_settings().mqtt_pass;

    // create the mqtt client
    let mqtt_client = mqtt::AsyncClient::new(mqtt_host).unwrap();
    let opts = mqtt::ConnectOptionsBuilder::new()
        .user_name(mqtt_user)
        .password(mqtt_pass)
        .finalize();

    mqtt_client.connect_with_callbacks(opts, on_connect_success, on_connect_failure);

    // Set a closure to be called whenever the client connection is established.
    mqtt_client.set_connected_callback(|_cli: &mqtt::AsyncClient| {
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
                check_msg_topic(topic, &payload_str, zone.1);
            }
            // Check if topic is sqlsprinkler_system/command
            check_system_command_topic(topic, payload_str);
        }
    });

    loop {
        thread::sleep(Duration::from_millis(5000));
        // Send current status of all zones
        for zone in get_zones().zones {
            // send current status
            let mut topic = format!("sqlsprinkler_zone_{}/status", zone.id);
            let mut payload = if zone.get_with_state().state {
                "ON"
            } else {
                "OFF"
            };
            let mut msg = mqtt::Message::new(topic, payload, 0);
            mqtt_client.publish(msg);

            // send current time
            topic = format!("sqlsprinkler_zone_{}_time/status", zone.id);
            // Set payload to zone.time
            let tmp_payload = format!("{}", zone.time);
            payload = tmp_payload.as_str();
            msg = mqtt::Message::new(topic, payload, 0);
            mqtt_client.publish(msg);

            // send current auto off state
            topic = format!("sqlsprinkler_zone_{}_auto_off_state/status", zone.id);
            payload = if zone.auto_off { "ON" } else { "OFF" };
            msg = mqtt::Message::new(topic, payload, 0);
            mqtt_client.publish(msg);

            // send current enabled state
            topic = format!("sqlsprinkler_zone_{}_enabled_state/status", zone.id);
            payload = if zone.enabled { "ON" } else { "OFF" };
            msg = mqtt::Message::new(topic, payload, 0);
            mqtt_client.publish(msg);
        }
        // send current status of the system switch
        let topic = "sqlsprinkler_system/status";
        let payload = if get_system_status() { "ON" } else { "OFF" };
        let msg = mqtt::Message::new(topic, payload, 0);
        mqtt_client.publish(msg);
    }
}

/// Checks if the topic is eqal to the system command topic - sqlsprinkler_system/command
fn check_system_command_topic(topic: &str, payload_str: Cow<str>) {
    if topic == "sqlsprinkler_system/command" {
        set_system_status(payload_str == "ON");
    }
}

/// Checks if the given topic matches the zone topic, zone time topic, zone auto off topic, or zone enabled topic
/// # Arguments
/// * `topic` - topic to check
/// * `payload_str` - payload to check
/// * `zone` - The sprinkler zone to check
fn check_msg_topic(topic: &str, payload_str: &str, zone: &Zone) {
    check_msg_zone(topic, payload_str, zone);
    // check if the payload matches sqlsprinkler_zone_<zone_id>_time
    check_msg_time(topic, payload_str, zone);
    // check if payload matches sqlsprinkler_zone_<zone_id>_auto_off_state
    check_msg_autooff(topic, payload_str, zone);
    // check if payload matches sqlsprinkler_zone_<zone_id>_enabled_state
    check_msg_enabled_state(topic, payload_str, zone)
}

/// Checks if the given topic matches the zone enabled topic - sqlsprinkler_zone_<zone_id>_enabled_state/command
/// # Arguments
/// * `topic` - topic to check
/// * `payload_str` - payload to check
/// * `zone` - The sprinkler zone to check
fn check_msg_enabled_state(topic: &str, payload_str: &str, zone: &Zone) {
    if topic == format!("sqlsprinkler_zone_{}_enabled_state/command", zone.id) {
        let enabled_state = payload_str.parse::<bool>().unwrap();
        let mut new_zone = zone.clone();
        new_zone.enabled = enabled_state;
        zone.update(new_zone);
    }
}

/// Checks if the given topic matches the zone auto off topic - sqlsprinkler_zone_<zone_id>_auto_off_state/command
/// # Arguments
/// * `topic` - topic to check
/// * `payload_str` - payload to check
/// * `zone` - The sprinkler zone to check
fn check_msg_autooff(topic: &str, payload_str: &str, zone: &Zone) {
    if topic == format!("sqlsprinkler_zone_{}_auto_off_state/command", zone.id) {
        let auto_off_state = payload_str.parse::<bool>().unwrap();
        let mut new_zone = zone.clone();
        new_zone.auto_off = auto_off_state;
        zone.update(new_zone);
    }
}

/// Checks if the given topic matches the zone time topic - sqlsprinkler_zone_<zone_id>_time/command
/// # Arguments
/// * `topic` - topic to check
/// * `payload_str` - payload to check
/// * `zone` - The sprinkler zone to check
fn check_msg_time(topic: &str, payload_str: &str, zone: &Zone) {
    if topic == format!("sqlsprinkler_zone_{}_time/command", zone.id) {
        let time = payload_str.parse::<u64>().unwrap();
        let mut new_zone = zone.clone();
        new_zone.time = time;
        zone.update(new_zone);
    }
}

/// Checks if the given topic matches the zone topic - sqlsprinkler_zone_<zone_id>/command
/// # Arguments
/// * `topic` - topic to check
/// * `payload_str` - payload to check
/// * `zone` - The sprinkler zone to check
fn check_msg_zone(topic: &str, payload_str: &str, zone: &Zone) {
    if topic == format!("sqlsprinkler_zone_{}/command", zone.id) {
        turn_off_all_zones();
        if payload_str == "ON" {
            zone.turn_on();
        }
    }
}

/// The callback method for when the client successfully connects to the MQTT broker.
/// This method will broadcast the home assistant discovery message.
/// # Arguments
/// * `cli` - The MQTT client
/// * `_msgid` - The message ID (unused)
fn on_connect_success(cli: &mqtt::AsyncClient, _msgid: u16) {
    println!("Connection succeeded");

    //broadcast all of the zones.
    for zone in get_zones().zones {
        // Broadcast the zone discovery message to the MQTT broker (as a switch)
        let mut zone_topic = format!("homeassistant/switch/sqlsprinkler_zone_{}/config", &zone.id);
        let mut mqtt_sprinkler = mqttdevice::MqttDevice::sprinkler(&zone);
        let mut payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
        let mut msg = mqtt::Message::new(zone_topic.clone(), payload.clone(), 0);
        println!("Sending MQTT message: {}", payload.clone());
        cli.publish(msg);
        ZONES
            .write()
            .unwrap()
            .insert(mqtt_sprinkler.cmd_t, zone.clone());

        // Broadcast the zone time to the mqtt broker (as a number)
        zone_topic = format!(
            "homeassistant/number/sqlsprinkler_zone_{}_time/config",
            &zone.id
        );
        mqtt_sprinkler = mqttdevice::MqttDevice::zone_time(&zone);
        payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
        msg = mqtt::Message::new(zone_topic.clone(), payload.clone(), 0);
        println!("Sending MQTT message: {}", payload.clone());
        cli.publish(msg);
        ZONES
            .write()
            .unwrap()
            .insert(mqtt_sprinkler.cmd_t, zone.clone());

        // Broadcast the zone auto off state to the mqtt broker (as a switch)
        zone_topic = format!(
            "homeassistant/switch/sqlsprinkler_zone_{}_auto_off/config",
            &zone.id
        );
        mqtt_sprinkler = mqttdevice::MqttDevice::zone_auto_off(&zone);
        payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
        msg = mqtt::Message::new(zone_topic.clone(), payload.clone(), 0);
        println!("Sending MQTT message: {}", payload.clone());
        cli.publish(msg);
        ZONES
            .write()
            .unwrap()
            .insert(mqtt_sprinkler.cmd_t, zone.clone());

        // Broadcast the zone enabled state to the mqtt broker (as a switch)
        zone_topic = format!(
            "homeassistant/switch/sqlsprinkler_zone_{}_enabled/config",
            &zone.id
        );
        mqtt_sprinkler = mqttdevice::MqttDevice::zone_enabled(&zone);
        payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
        msg = mqtt::Message::new(zone_topic.clone(), payload.clone(), 0);
        println!("Sending MQTT message: {}", payload.clone());
        cli.publish(msg);
        ZONES
            .write()
            .unwrap()
            .insert(mqtt_sprinkler.cmd_t, zone.clone());
    }

    // broadcast the system toggle
    let topic = "homeassistant/switch/sqlsprinkler_system/config";
    let system = mqttdevice::MqttDevice::system();
    let payload = serde_json::to_string(&system).unwrap();
    let msg = mqtt::Message::new(topic, payload.clone(), 0);
    ZONES.write().unwrap().insert(system.cmd_t, Zone::default());
    println!("Sending MQTT message: {}", payload);
    cli.publish(msg);
    // Subscribe to all zone topics
    for zone in ZONES.read().unwrap().keys() {
        cli.subscribe(zone, 0);
        println!("Subscribed to {}", zone);
    }
}

/// Callback for a failed attempt to connect to the server.
/// We simply sleep and then try again.
/// # Arguments
/// * `cli` - The mqtt client
/// * `_msgid` - The message id (unused)
/// * `rc` - The reason for the failure
fn on_connect_failure(cli: &mqtt::AsyncClient, _msgid: u16, rc: i32) {
    println!("Connection attempt failed with error code {}.\n", rc);
    thread::sleep(Duration::from_millis(2500));
    cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
}
