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

## Dependencies
* rust >= 1.53.0
* structopt 0.3.13
* mysql 16.1.0
* serde_json
* tokio 1.0
* warp 0.3

## How-to-use
* Export your SQL password as an environment variable called `SQL_PASS`
  - ie, `export SQL_PASS='password123'`
* run your given sqlsprinkler command, and enjoy!