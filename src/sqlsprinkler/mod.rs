use log::error;
use std::process::exit;
use sqlx::{MySqlPool};

use crate::get_settings;

pub mod daemon;
pub mod system;
pub mod zone;

/// Gets a connection to a MySQL database
/// # Return
///     `Pool` A connection to the SQL database.
///
pub(crate) async fn get_pool() -> Result<MySqlPool, sqlx::Error> {
    // Build the url for the connection
    let reader = get_settings();

    if reader.sqlsprinkler_user.is_empty() {
        error!(
            "Missing configuration for sqlsprinkler_user in /etc/sqlsprinkler/sqlsprinkler.conf"
        );
        exit(1);
    }
    if reader.sqlsprinkler_pass.is_empty() {
        error!(
            "Missing configuration for sqlsprinkler_pass in /etc/sqlsprinkler/sqlsprinkler.conf"
        );
        exit(1);
    }
    if reader.sqlsprinkler_host.is_empty() {
        error!(
            "Missing configuration for sqlsprinkler_host in /etc/sqlsprinkler/sqlsprinkler.conf"
        );
        exit(1);
    }
    if reader.sqlsprinkler_db.is_empty() {
        error!("Missing configuration for sqlsprinkler_db in /etc/sqlsprinkler/sqlsprinkler.conf");
        exit(1);
    }

    let url = format!(
        "mysql://{}:{}@{}:3306/{}",
        reader.sqlsprinkler_user,
        reader.sqlsprinkler_pass,
        reader.sqlsprinkler_host,
        reader.sqlsprinkler_db
    );
    let pool = MySqlPool::connect(&url).await?;
    Ok(pool)
}
