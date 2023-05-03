use crate::mqtt::mqttdevice;
use crate::{
    get_settings, get_system_status, get_zones, set_system_status, turn_off_all_zones,
    Zone,
};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use paho_mqtt as mqtt;
use paho_mqtt::{AsyncClient};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::process::exit;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

lazy_static! {
    // create a hashmap of topics and zone
    static ref ZONES: RwLock<HashMap<String, Zone>> = RwLock::new(HashMap::new());
}

/// Runs the MQTT client for the sprinkler system.
/// Connects to the MQTT broker given in the settings, and authenticates with the username and password (also pulled from settings).
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
/// # Examples
/// ```
/// use sqlsprinkler::mqtt::mqtt_client;
/// mqtt_client::run();
/// ```
pub async fn mqtt_run() -> Result<(), Box<dyn Error>> {
    let mqtt_host = get_settings().mqtt_host;
    let mqtt_user = get_settings().mqtt_user;
    let mqtt_pass = get_settings().mqtt_pass;
    if mqtt_host.is_empty() {
        error!("MQTT host is not set!");
        exit(1);
    }
    if mqtt_user.is_empty() {
        error!("MQTT user is not set!");
        exit(1);
    }
    if mqtt_pass.is_empty() {
        error!("MQTT password is not set!");
        exit(1);
    }

    // create the mqtt client
    let mqtt_client = AsyncClient::new(mqtt_host).unwrap();
    let opts = mqtt::ConnectOptionsBuilder::new()
        .user_name(mqtt_user)
        .password(mqtt_pass)
        .finalize();

    info!("Connecting to MQTT broker...");
    mqtt_client.connect_with_callbacks(opts, on_connect_success, on_connect_failure);

    // Set a closure to be called whenever the client connection is established.
    mqtt_client.set_connected_callback(|_cli: &AsyncClient| {
        info!("Connected.");
    });

    // Set a closure to be called whenever the client loses the connection.
    // It will attempt to reconnect, and set up function callbacks to keep
    // retrying until the connection is re-established.
    mqtt_client.set_connection_lost_callback(|cli: &AsyncClient| {
        warn!("Connection lost. Attempting reconnect.");
        thread::sleep(Duration::from_millis(2500));
        cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
    });

    // Start listening for mqtt messages
    mqtt_client.set_message_callback(|cli, msg| {
        if let Some(msg) = msg {
            let topic = msg.topic();
            let payload_str = msg.payload_str();
            debug!("listening on {} - {}", topic, payload_str);
            for zone in ZONES.read().unwrap().iter() {
                if check_msg_topic(cli, topic, &payload_str, zone.1) {
                    break;
                }
            }
            check_system_command_topic(topic, payload_str);
        }
    });

    let mut loop_count = 0;
    loop {
        thread::sleep(Duration::from_millis(5000));
        // Send current status of all zones
        for zone in get_zones().await?.zones {
            // send current status
            let mut topic = format!("sqlsprinkler_zone_{}/status", zone.id);
            let mut payload = if zone.get_with_state().state {
                "ON"
            } else {
                "OFF"
            };
            send_payload(&mqtt_client, &topic, payload, false);

            // send current time
            topic = format!("sqlsprinkler_zone_{}_time/status", zone.id);
            // Set payload to zone.time
            let tmp_payload = format!("{}", zone.Time);
            payload = tmp_payload.as_str();
            send_payload(&mqtt_client, &topic, payload, false);

            // send current auto off state
            topic = format!("sqlsprinkler_zone_{}_auto_off_state/status", zone.id);
            payload = if zone.Autooff { "ON" } else { "OFF" };
            send_payload(&mqtt_client, &topic, payload, false);

            // send current enabled state
            topic = format!("sqlsprinkler_zone_{}_enabled_state/status", zone.id);
            payload = if zone.Enabled { "ON" } else { "OFF" };
            send_payload(&mqtt_client, &topic, payload, false);
        }
        // send current status of the system switch
        let topic = "sqlsprinkler_system/status";
        let payload = if get_system_status().await? { "ON" } else { "OFF" };
        send_payload(&mqtt_client, topic, payload, false);

        loop_count = (loop_count + 1) % 15;
        if loop_count == 0 {
            // broadcast
            send_discovery_message(&mqtt_client);
        }
    }
    Ok(())
}


fn send_payload(mqtt_client: &AsyncClient, topic: &str, payload: &str, retain: bool) {
    let msg = mqtt::MessageBuilder::new()
        .topic(topic)
        .payload(payload)
        .retained(retain)
        .qos(0)
        .finalize();
    debug!("sending payload {}", payload);
    mqtt_client.publish(msg);
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
fn check_msg_topic(cli: &AsyncClient, topic: &str, payload_str: &str, zone: &Zone) -> bool {
    if topic == format!("sqlsprinkler_zone_{}/command", zone.id).as_str() {
        turn_off_all_zones();
        if payload_str == "ON" {
            zone.turn_on()
        }
        // send current status
        let _topic = format!("sqlsprinkler_zone_{}/status", zone.id);
        send_payload(&cli, &_topic, payload_str, false);
        info!("Zone {} turned {}", zone.id, payload_str);
        return true;
    } else if topic == format!("sqlsprinkler_zone_{}_time/command", zone.id).as_str() {
        let time = payload_str.parse::<u32>().unwrap();
        let mut _zone = zone.clone();
        _zone.Time = time as i64;
        zone.update(_zone);
        // send current time
        let _topic = format!("sqlsprinkler_zone_{}_time/status", zone.id);
        send_payload(&cli, &_topic, payload_str, false);
        info!("Zone {} time set to {}", zone.id, time);
        return true;
    } else if topic == format!("sqlsprinkler_zone_{}_auto_off_state/command", zone.id).as_str() {
        let auto_off = payload_str == "ON";
        let mut _zone = zone.clone();
        _zone.Autooff = auto_off;
        zone.update(_zone);
        // send current auto off state
        let _topic = format!("sqlsprinkler_zone_{}_auto_off_state/status", zone.id);
        send_payload(&cli, &_topic, payload_str, false);
        info!("Zone {} auto off state set to {}", zone.id, auto_off);
        return true;
    } else if topic == format!("sqlsprinkler_zone_{}_enabled_state/command", zone.id).as_str() {
        let enabled = payload_str == "ON";
        let mut _zone = zone.clone();
        _zone.Enabled = enabled;
        zone.update(_zone);
        // send current enabled state
        let _topic = format!("sqlsprinkler_zone_{}_enabled_state/status", zone.id);
        send_payload(&cli, &_topic, payload_str, false);
        info!("Zone {} enabled state changed to {}", zone.id, enabled);
        return true;
    } else if topic == "sqlsprinkler_system/command" {
        set_system_status(payload_str == "ON");
        // send current status
        let _topic = "sqlsprinkler_system/status";
        send_payload(&cli, &_topic, payload_str, false);
        info!("System command {}", payload_str);
        return true;
    }
    false
}

/// The callback method for when the client successfully connects to the MQTT broker.
/// This method will broadcast the home assistant discovery message.
/// # Arguments
/// * `cli` - The MQTT client
/// * `_msgid` - The message ID (unused)
fn on_connect_success(cli: &AsyncClient, _msgid: u16) {
    info!("Connection succeeded");
    send_discovery_message(cli).unwrap();
    // Subscribe to all zone topics
    for zone in ZONES.read().unwrap().keys() {
        cli.subscribe(zone, 0);
        info!("Subscribed to {}", zone);
    }
}

/// Sends the discovery message of all of the entities to the MQTT broker.
fn send_discovery_message(cli: &AsyncClient) -> Result<(), Box<dyn Error + '_>> {
    //broadcast all of the zones.
    // block on getting the zones

    let m_cli = cli.clone();
    thread::spawn(move || async move {
        for zone in get_zones().await.unwrap().zones {
            // Broadcast the zone discovery message to the MQTT broker (as a switch)
            let mut zone_topic = format!("homeassistant/switch/sqlsprinkler_zone_{}/config", &zone.id);
            let mut mqtt_sprinkler = mqttdevice::MqttDevice::sprinkler(&zone);
            let payload = serde_json::to_string(&mqtt_sprinkler).unwrap();
            send_payload(&m_cli, &zone_topic, &payload, true);
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
            let payload2 = serde_json::to_string(&mqtt_sprinkler).unwrap();
            send_payload(&m_cli, &zone_topic, &payload2, true);
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
            let payload3 = serde_json::to_string(&mqtt_sprinkler).unwrap();
            send_payload(&m_cli, &zone_topic, &payload3, true);
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
            let payload4 = serde_json::to_string(&mqtt_sprinkler).unwrap();
            send_payload(&m_cli, &zone_topic, &payload4, true);

            ZONES
                .write()
                .unwrap()
                .insert(mqtt_sprinkler.cmd_t, zone.clone());
        }

        // broadcast the system toggle
        let topic = "homeassistant/switch/sqlsprinkler_system/config";
        let system = mqttdevice::MqttDevice::system();
        let payload = serde_json::to_string(&system).unwrap();
        ZONES.write().unwrap().insert(system.cmd_t, Zone::default());
        send_payload(&m_cli, topic, &payload, true);
    });
    Ok(())
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
