use std::thread;
use std::time::Duration;
use crate::{get_settings, get_system_status, get_zones, serde_json, set_system_status, turn_off_all_zones, Zone};
use paho_mqtt as mqtt;
use std::sync::RwLock;
use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::mqtt::mqttsprinkler;


lazy_static! {
    // create a hashset of topics and zone
    static ref ZONES: RwLock<HashMap<String, Zone>> = RwLock::new(HashMap::new());
}

pub fn mqtt_run() -> ! {
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
            let mut payload = format!("{}", if zone.get_zone_with_state().state { "ON" } else { "OFF" });
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