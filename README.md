# sqlsprinkler-cli 0.1.1

A command line interface for the SQLSprinkler project

## Authors & Contributers

- Gavin Pease

## Usage

sqlsprinkler-cli allows control over a SQLSprinkler endpoint via a unified program.

* `sqlsprinkler-cli`
    - Prints out help & version information
* `sqlsprinkler-cli --daemon`
    - Starts the SQLSprinkler daemon
* `sqlsprinkler-cli zone <id> <on,off,status>`
    - Turn the given zone on or off
* `sqlsprinkler-cli sys <on,off,winterize,run,status>`
    - Operate on the system.

## TODO
* [ ] Implement `sqlsprinkler-cli sys winterize (test?)`
    *   Run each system for 10 seconds, and then sleep for 3 minutess
* [ ] A and B days

## Features added
* [x] `sqlsprinkler-cli sys run`
* [x] `sqlsprinkler-cli zone <id> <on,off,status>`
* [x] `sqlsprinkler-cli sys <on,off>`
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

## How-to-use

* Export your SQL password, user, and host as environment variables.
    - ie, `export SQL_PASS='password123' ; export SQL_HOST='host' ; export SQL_USER='user'`
* run your wanted sqlsprinkler command, and enjoy!