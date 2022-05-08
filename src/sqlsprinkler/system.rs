use crate::get_settings;
use crate::sqlsprinkler::{get_pool, zone};
use log::info;
use std::{thread, time};

#[derive(Debug, PartialEq, Eq)]
pub struct SysStatus {
    status: bool,
}

/// Enables or disables the system schedule
/// # Arguments
///     * `enabled` If true is passed in, the system is enabled. If false is used, the system is disabled.
/// # Example
/// ```
/// use sqlsprinkler::system::set_status;
/// set_status(true);
/// ```
pub fn set_system_status(enabled: bool) {
    let pool = get_pool();
    let query = "UPDATE Enabled set enabled = ?";
    pool.prep_exec(query, (enabled,)).unwrap();
}

/// Gets whether the system schedule is enabled or disabled
/// # Return
/// A bool representing whether the system is enabled or disabled.
/// # Example
/// ```
/// use sqlsprinkler::system::get_system_status;
/// let status = get_system_status();
/// ```
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
/// # Returns
///     A `ZoneList` containing all the zones ordered by their system order.
/// # Example
/// ```
/// use sqlsprinkler::system;
/// let zones = system::get_zones();
/// ```
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
    }
    info!("Got {} zones", zone_list.len());
    zone::ZoneList { zones: zone_list }
}

/// Runs the system based on the schedule configured. Skips over any zones that are not enabled in the database.
/// # Example
/// ```
/// use sqlsprinkler::system;
/// system::run_system();
/// ```
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
/// # Example
/// ```
/// use sqlsprinkler::system;
/// system::turn_off_all_zones();
/// ```
pub(crate) fn turn_off_all_zones() {
    info!("Turning off all zones");
    let zone_list = get_zones();
    for zone_in_list in &zone_list.zones {
        zone_in_list.turn_off();
    }
}

/// Winterizes the system by turning on a zone for a minute, followed by a three minute delay.
/// # Example
/// ```
/// use sqlsprinkler::system;
/// system::winterize();
/// ```
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
