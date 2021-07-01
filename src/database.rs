use mysql::*;
use mysql::prelude::*;

#[derive(Debug, PartialEq, Eq)]
struct Zone {
    name: String,
    gpio: i8,
    time: i32,
    enabled: bool,
    auto_off: bool,
    system_order: i8,
}

