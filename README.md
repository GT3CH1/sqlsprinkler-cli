# sqlsprinkler-cli

### v0.1.6

A command line interface for the SQLSprinkler project, made specifically for the raspberry pi.

## Authors & Contributors

- Gavin Pease

## Building

* Dependencies
    - [cross](https://github.com/rust-embedded/cross)
    - Docker
    - Rust

After installing the build dependencies, you can run `make deb` to create a `.deb` package, you can
then install it with `sudo dpkg -i sqlsprinkler-cli_0.1.6_armhf.deb`.

## Installing

* To install, please run `# make install`. This will install the binary to `/usr/bin/sqlsprinkler-cli`.
    * Running this command _should_ generate a binary that will work across all raspberry pi's.

## Usage

sqlsprinkler-cli allows control over a SQLSprinkler endpoint via a unified program.

* `sqlsprinkler-cli`
    - Prints out help & version information
* `sqlsprinkler-cli --daemon`, `sqlsprinkler -w`
    - Starts the SQLSprinkler daemon on port 3030.
* `sqlsprinkler-cli zone <id> <on,off,status>`
    - Turn the given zone on or off
* `sqlsprinkler-cli zone add <name> <gpio> <time> <enabled> <auto off>`
    - Add a zone to the system.
* `sqlsprinkler-cli zone delete <id>`
    - Delete a zone from the system.
* `sqlsprinkler-cli zone modify <id> <name> <gpio> <time> <enabled> <auto off>`
    - Modifys a zone in the system.
* `sqlsprinkler-cli sys <on,off,winterize,run,status>`
    - Operate on the system.
* `sqlsprinkler-cli -m`
    - Starts the SQLSprinkler MQTT listener for home assistant integration.
* You can set the database username, password, and host in the `/etc/sqlsprinkler/sqlsprinkler.conf` configuration file.

## TODO

* [ ] Create tables and databases if they do not exist.
* [ ] A and B days
* [ ] Make `sqlsprinkler zone ...` call the Web API to control turning zones on and off.
* [x] Better error messages

## Features added

* [x] A configuration file at `/etc/sqlsprinkler/sqlsprinkler.conf`
* [x] `sqlsprinkler-cli sys test`
* [x] `sqlsprinkler-cli sys run`
* [x] `sqlsprinkler-cli zone <id> <on,off,status>`
* [x] `sqlsprinkler-cli sys <on,off>`
* [x] SQLSprinkler web api
  * Please see the [API Documentation](#api-documentation) for more information.
    * [x] Get system schedule status → `GET /system/status`
    * [x] Update system schedule status → `PUT /system/status ` → `{"system_status": status}`
    * [x] Get all zones status → `GET /zone/info`
    * [x] Get single zone status → `GET /zone/info/<id>`
    * [x] Toggle zone → `PUT /zone` → `{"id": id, "state": state}`
    * [x] Update zone information → `PUT /zone/info` → `{
      "Name": "Rust-Zone 123",
      "GPIO": 12,
      "Time": 10,
      "Autooff": true,
      "Enabled": true,
      "SystemOrder": 1,
      "id": 4 }`
    * [x] Create zone → `POST /zone` → `{
      "Name": "Rust-Zone",
      "Gpio": 12,
      "Time": 10,
      "Autooff": true,
      "Enabled": true }`
    * [x] Delete zone → ` DELETE /zone` → `{
      "id": 1 }`
    * [x] Change zone ordering → `PUT /zone/order` → `{"order":[0,0,0]}`
    * Home Assistant support can be found in the `sqlsprinkler-ha`
      repository, [here](https://github.com/GT3CH1/sqlsprinkler-ha).

## Used libraries

* rust >= 1.53.0
* structopt 0.3.13
* mysql 16.1.0
* tokio 1.0
* warp 0.3
* parking_lot 0.10.0
* rppal 0.12.0
* chrono 0.3.0
* confy 0.4.0
* lazy_static 1.4.0
* paho-mqtt 0.11
* serde 1.0

## How-to-use

* Run the program once, as `sudo`, you will get a connection error.
* Set your username, password, host, and database in `/etc/sqlsprinkler/sqlsprinkler.conf`
    * if you are using mqtt, then please set the `mqtt_host`, `mqtt_pass`, and `mqtt_user` in the configuration.
* run your wanted sqlsprinkler command, and enjoy!

## About the config

- The settings prefixed with `sqlsprinkler_` should be pretty self explanitory.
- `verbose` Possible values: true/false → enables verbose logging.
- `mqtt_host` The hostname of the mqtt broker.
- `mqtt_user` The username of the mqtt broker.
- `mqtt_pass` The password of the mqtt broker.

## Issues and bugs

* Please report any issues or bugs [here](https://github.com/GT3CH1/sqlsprinkler-cli/issues)

### License

This project is licensed under the MIT license. Please read the [LICENSE](LICENSE) file for more information.

### Contributing

* Please feel free to by:
    - Making an issue [here](https://github.com/GT3CH1/sqlsprinkler-cli/issues)
    - Forking the project [here](https://github.com/GT3CH1/sqlsprinkler-cli)
    - Making changes to the project.
    - Opening a pull request [here](https://github.com/GT3CH1/sqlsprinkler-cli/pulls)
    - Lastly, please be descriptive and constructive with your contribution.

### Code of conduct
* Please read the [CODE OF CONDUCT](CODE_OF_CONDUCT) file for more information.

## API Documentation
### Getting the system state
```http request
GET /system/state
```
#### Response
```json
{
  "system_enabled": true
}
```
---
### Updating the system state
```http request
PUT /system/state
```

#### Payload
```json
{
  "system_enabled": false
}
```
Setting the system state to false will disable the system, where as setting it to true will enable the system.

---

#### Getting information for all zones
```http request
GET /zone/info
```
#### Response
```json
[
    {
        "Name": "Rust-Zone 1",
        "gpio": 12,
        "time": 10,
        "enabled": true,
        "auto_off": true,
        "system_order": 0,
        "state": false,
        "id": 1
    }
    ...
]
```

This will return a list of all the zones and their information.

---

### Updating the state of a zone
```http request
PUT /zone
```
#### Payload
```json
{
  "id": 1,
  "state": true
}
```
This will turn on a zone with the ID of 1.

---

### Adding a zone
```http request
POST /zone
```
#### Payload
```json
{
  "name": "Rust-Zone",
  "gpio": 12,
  "time": 10,
  "enabled": true,
  "auto_off": true,
}
```

This will add a zone with the name of "Rust-Zone", GPIO pin 12, time 10 minutes, enabled, and auto off.
System order and ID aren't specified. The ID will be automatically assigned, and the system order will be set to the default of 0.

---

### Deleting a zone
```http request
DELETE /zone
```
#### Payload
```json
{
  "id": 1
}
```

This will delete the zone with the ID of 1.

---

### Updating zone information
```http request
PUT /zone/update
```
#### Payload
```json
{
  "id": 1,
  "name": "Rust-Zone",
  "gpio": 12,
  "time": 10,
  "enabled": true,
  "auto_off": true,
  "system_order": 0,
}
```

This will update the zone with a matching ID with the information provided.

---

#### Updating zone order
```http request
PUT /zone/order
```
#### Payload
```json
{
  "order" : [0,4,3,2]
}
```

This will update the order of the zones, updating the zones ORDERED BY the current system order.
This example would mean the zone that is currently at system order 0, will be moved to system order 0,
the zone that is currently at system order 1, will be moved to system order 4, and so on.
=======

* Please read the [CODE OF CONDUCT](CODE_OF_CONDUCT) file for more information.
