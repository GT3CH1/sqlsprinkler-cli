use crate::sqlsprinkler::{get_pool, zone};
use log::{error, info};
use std::{thread, time};
use std::error::Error;
use crate::sqlsprinkler::zone::Zone;

#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct SysStatus {
    status: bool,
}

/// Enables or disables the system schedule
/// # Arguments
///     * `enabled` If true is passed in, the system is enabled. If false is used, the system is disabled.
/// # Example
/// ```
/// use sqlsprinkler::system::set_system_status;
/// set_system_status(true);
/// ```
pub async fn set_system_status(enabled: bool) -> Result<(), Box<dyn Error>> {
    // let pool = &get_pool();
    // let query = "UPDATE Enabled set enabled = ?";
    // pool.prep_exec(query, (enabled, )).unwrap();
    sqlx::query!("UPDATE Enabled set enabled = ?", enabled)
        .execute(&get_pool())
        .await?;
    info!("System status set to {}", enabled);
    Ok(())
}

/// Gets whether the system schedule is enabled or disabled
/// # Return
/// A bool representing whether the system is enabled or disabled.
/// # Example
/// ```
/// use sqlsprinkler::system::get_system_status;
/// let status = get_system_status();
/// ```
pub(crate) async fn get_system_status() -> Result<bool, sqlx::Error> {
    let rows = sqlx::query_as::<_, SysStatus>("SELECT enabled as status from Enabled")
        .fetch_all(&get_pool()).await?;
    Ok(rows[0].status)
}

/// Gets a list of all the zones in this database
/// # Returns
///     A `ZoneList` containing all the zones ordered by their system order.
/// # Example
/// ```
/// use sqlsprinkler::system;
/// let zones = system::get_zones();
/// ```
pub(crate) async fn get_zones() -> Result<zone::ZoneList, sqlx::Error> {
    let mut rows = sqlx::query_as::<_, Zone>("SELECT * FROM Zones ORDER BY SystemOrder")
        .fetch_all(&get_pool()).await?;
    let mut res = vec![];
    for row in rows.iter_mut() {
        res.push(row.clone());
    }
    Ok(zone::ZoneList { zones: res })
}

/// Runs the system based on the schedule configured. Skips over any zones that are not enabled in the database.
/// # Example
/// ```
/// use sqlsprinkler::system;
/// system::run();
/// ```
pub async fn run() -> Result<(), Box<dyn Error>> {
    let zone_list = get_zones().await?;
    info!("Running system as scheduled");
    for zone in &zone_list.zones {
        // Skip over zones that aren't enabled in the database.
        if zone.Enabled {
            zone.run();
        }
    }
    info!("System run complete");
    Ok(())
}

/// Turns off all the zones in the system
/// # Example
/// ```
/// use sqlsprinkler::system;
/// system::turn_off_all_zones();
/// ```
pub(crate) async fn turn_off_all_zones() -> Result<bool, rppal::gpio::Error> {
    info!("Turning off all zones");
    let zone_list = get_zones().await.unwrap();
    for zone_in_list in &zone_list.zones {
        zone_in_list.turn_off();
    }
    Ok(true)
}

/// Winterizes the system by turning on a zone for a minute, followed by a three minute delay.
/// # Example
/// ```
/// use sqlsprinkler::system;
/// system::winterize();
/// ```
pub(crate) async fn winterize() -> Result<(), Box<dyn Error>> {
    let zone_list = get_zones().await?;
    for zone in &zone_list.zones {
        info!("Winterizing zone {}", zone.Name);
        zone.turn_on();
        let _zone = zone.clone();
        let run_time = time::Duration::from_secs(60);
        thread::sleep(run_time);
        _zone.turn_off();
        let run_time = time::Duration::from_secs(3 * 60);
        thread::sleep(run_time);
        info!("Winterized zone {}", zone.Name);
    }
    Ok(())
}
