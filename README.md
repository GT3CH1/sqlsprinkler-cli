# sqlsprinkler-cli 0.1.2

A command line interface for the SQLSprinkler project

## Authors & Contributers

- Gavin Pease

## Building
* Dependencies
    - [cross](https://github.com/rust-embedded/cross)
    - Docker

After installing dependencies run `make build-rpi` to build the program.

## Installing
* To install, please run `# make install`
* To install the service, please run `#make install-service` (`make install` will also do this).

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
* You can set the database username, password, and host in the `/etc/sqlsprinkler/sqlsprinkler.conf` configuration file.

## TODO
* [ ] Create tables and databases if they do not exist.
* [ ] Implement `sqlsprinkler-cli sys winterize (test?)`
    *   Run each system for 10 seconds, and then sleep for 3 minutess
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


## Dependencies

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

## How-to-use

* Run the program once, as `sudo`, you will get a connection error.
* Set your username, password, host, and database in `/etc/sqlsprinkler/sqlsprinkler.conf`
* run your wanted sqlsprinkler command, and enjoy!

## About the config
- The settings prefixed with `sqlsprinkler_` should be pretty self explanitory.
- `verbose` Possible values: true/false → enables verbose logging.