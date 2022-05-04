# sqlsprinkler-cli
### v0.1.5
A command line interface for the SQLSprinkler project, made specifically for the raspberry pi.

## Authors & Contributors
- Gavin Pease

## Building
* Dependencies
    - [cross](https://github.com/rust-embedded/cross)
    - Docker
    - Rust

After installing the build dependencies, you can run `make deb` to create a `.deb` package, you can
then install it with `sudo dpkg -i sqlsprinkler-cli_0.1.5_armhf.deb`.

## Installing
* To install, please run `# make install`. This will install the binary to `/usr/bin/sqlsprinkler-cli`.
    * Running this command _should_ generate a binary that will work across all raspberry pi's.

## Usage
sqlsprinkler-cli allows control over a SQLSprinkler endpoint via a unified program.

* `sqlsprinkler-cli`
    - Prints out help & version information
* `sqlsprinkler-cli --daemon`
    - Starts the SQLSprinkler daemon on port 3030.
* `sqlsprinkler-cli zone <id> <on,off,status>`
    - Turn the given zone on or off
* `sqlsprinkler-cli sys <on,off,winterize,run,status>`
    - Operate on the system.
* `sqlsprinkler-cli -m`
    - Starts the SQLSprinkler MQTT listener for home assistant integration.
* You can set the database username, password, and host in the `/etc/sqlsprinkler/sqlsprinkler.conf` configuration file.

## TODO
* [ ] Create tables and databases if they do not exist.
* [ ] A and B days
* [ ] Make `sqlsprinkler zone ...` call the Web API to control turning zones on and off.
* [ ] Better error messages

## Features added
* [x] A configuration file at `/etc/sqlsprinkler/sqlsprinkler.conf` 
* [x] `sqlsprinkler-cli sys test`
* [x] `sqlsprinkler-cli sys run`
* [x] `sqlsprinkler-cli zone <id> <on,off,status>`
* [x] `sqlsprinkler-cli sys <on,off>`
* [x] SQLSprinkler web api
    * [x] Get system schedule status → `GET /system/status`
    * [x] Update system schedule status → `PUT /system/status ` → `{"system_status": status}`
    * [x] Get zone status → `GET /zone/info`
    * [x] Toggle zone → `PUT /zone` → `{"id": id, "state": state}`
    * [x] Update zone information → `PUT /zone/info` → `{
      "name": "Rust-Zone 123",
      "gpio": 12,
      "time": 10,
      "auto_off": true,
      "enabled": true,
      "system_order": 1,
      "id": 4 }`
    * [x] Create zone → `POST /zone` → `{
      "name": "Rust-Zone",
      "gpio": 12,
      "time": 10,
      "auto_off": true,
      "enabled": true }`
    * [x] Delete zone → ` DELETE /zone` → `{
      "id": 1 }`
    * [x] Change zone ordering → `PUT /zone/order` → `{"order":[0,0,0]}`
* [x] Add support for MQTT and home assistant.
  * Topics subscribed to: 
    * `sqlsprinkler_zone_<id><_enabled_state,_time,_auto_off_state>/command`
      * Messages are just basic ON/OFF or numbers
  * Topics published to:
    * `homeassistant/switch/sqlsprinkler_zone_<id>/config`
      * Basic zone on/off functionality.
    * `homeassistant/number/sqlsprinkler_zone_<id>_time/config`
      * Time to run zone (in minutes).
    * `homeassistant/switch/sqlsprinkler_zone_<id>_auto_off/config`
      * Auto off functionality.
    * `homeassistant/switch/sqlsprinkler_system_<id>_enabled/config`
      * Zone enabled/disabled functionality.
    * `homeassistant/switch/sqlsprinkler_system/config`
      * The master toggle switch for nightly runs.
    * `sqlsprinkler_zone_<id><_enabled_state,_time,_auto_off_state>/status`
      * The status of the functionality of the zone.


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