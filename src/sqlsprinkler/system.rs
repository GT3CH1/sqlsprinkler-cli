use crate::get_settings;
use crate::sqlsprinkler::{get_pool, zone};
use std::{thread, time};

#[derive(Debug, PartialEq, Eq)]
pub struct SysStatus {
    status: bool,
}

/// Enables or disables the system schedule
/// # Arguments
///     * `pool` The SQL connection pool
/// used to toggle the system.
///     * `enabled` If true is passed in, the system is enabled. If false is used, the system is disabled.
pub fn set_system_status(enabled: bool) {
    let pool = get_pool();
    let query = "UPDATE Enabled set enabled = ?";
    pool.prep_exec(query, (enabled,)).unwrap();
}

/// Gets whether the system schedule is enabled or disabled
/// # Arguments
///     * `pool` The SQL connection pool used to toggle the system.
/// # Return
///     * `bool` True if the system is enabled, false if not.
pub(crate) fn get_system_status() -> bool {
    let pool = get_pool();
    let query = "SELECT enabled FROM Enabled";
    let sys_status: Vec<SysStatus> = pool
        .prep_exec(query, ())
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let status = mysql::from_row(row);
                    SysStatus { status }
                })
                .collect()
        })
        .unwrap();
    sys_status[0].status
}

/// Gets a list of all the zones in this database
/// # Arguments
///     * `pool` The SQL connection pool to use to query for zones
/// # Returns
///     * `Vec<Zone>` A list of all the zones in the database.
pub(crate) fn get_zones() -> zone::ZoneList {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let query = "SELECT Name, GPIO, Time, Enabled, AutoOff, SystemOrder, ID from Zones ORDER BY SystemOrder";
    let rows = conn.query(query).unwrap();
    let mut zone_list: Vec<zone::Zone> = vec![];
    for row in rows {
        let _row = row.unwrap();

        let zone = zone::Zone::from(_row);
        zone_list.push(zone.clone());
        if get_settings().verbose {
            println!("{}", zone);
        }
    }
    zone::ZoneList { zones: zone_list }
}

/// Runs the system based on the schedule configured. Skips over any zones that are not enabled in the database.
pub fn run() {
    let zone_list = get_zones();
    for zone in &zone_list.zones {
        // Skip over zones that aren't enabled in the database.
        if zone.enabled {
            zone.run();
        }
    }
}

/// Turns off all the zones in the system
pub(crate) fn turn_off_all_zones() {
    let zone_list = get_zones();
    for zone_in_list in &zone_list.zones {
        zone_in_list.turn_off();
    }
}

/// Winterizes the system by turning on a zone for a minute, followed by a three minute delay.
pub(crate) fn winterize() {
    let zone_list = get_zones();
    for zone in &zone_list.zones {
        zone.turn_on();
        let _zone = zone.clone();
        let run_time = time::Duration::from_secs(60);
        thread::sleep(run_time);
        _zone.turn_off();
        let run_time = time::Duration::from_secs(3 * 60);
        thread::sleep(run_time);
    }
}
