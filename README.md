# sqlsprinkler-cli
A command line interface for the SQLSprinkler project

## Authors & Contributers
- Gavin Pease

## Usage
sqlsprinkler-cli allows control over a SQLSprinkler endpoint via a unified program.

* `sqlsprinkler-cli`
    - Prints out help & version information
* `sqlsprinkler-cli zone <id> <on,off>`
    - Turn the given zone on or off
* `sqlsprinkler-cli sys <on,off,winterize,run,status>`
    - Operate on the system.
    
## TODO
* [ ] Implement `sqlsprinkler-cli`
* [ ] Implement `sqlsprinkler-cli zone <id> <on,off>`
* [ ] Implement `sqlsprinkler-cli sys run`
* [ ] Implement `sqlsprinkler-cli sys winterize`
* [ ] Implement `sqlsprinkler-cli sys <on,off>`
* [ ] Implement SQLSprinkler web api
  * [x] Get system schedule status → `GET /system/status` 
  * [x] Update system schedule status → `PUT /system/status ` → `{"system_status": status}`
  * [x] Get zone status → `GET /zone/info`
  * [ ] Toggle zone → `PUT /zone` → `{"id": id, "state": state}`
    - Currently, partially implemented, no GPIO pins will be toggled as of 07-02-2021
  * [ ] Update zone information
  * [x] Create zone → `POST /zone` → `{
    "name": "Rust-Zone",
    "gpio": 12,
    "time": 10,
    "auto_off": true,
    "enabled": true
    }`
  * [ ] Delete zone
  * [ ] Change zone ordering

## Dependencies
* rust >= 1.53.0
* structopt 0.3.13
* mysql 16.1.0
* tokio 1.0
* warp 0.3
* parking_lot 0.10.0

## How-to-use
* Export your SQL password as an environment variable called `SQL_PASS`
  - ie, `export SQL_PASS='password123'`
* run your given sqlsprinkler command, and enjoy!