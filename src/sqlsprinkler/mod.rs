use std::process::exit;

use mysql::Pool;

use crate::get_settings;

pub mod zone;
pub mod system;
pub mod daemon;

/// Gets a connection to a MySQL database
/// # Return
///     `Pool` A connection to the SQL database.
pub(crate) fn get_pool() -> Pool {
    // Build the url for the connection
    let reader = get_settings();
    let url = format!("mysql://{}:{}@{}:3306/{}",
                      reader.sqlsprinkler_user, reader.sqlsprinkler_pass, reader.sqlsprinkler_host, reader.sqlsprinkler_db);

    let pool = match mysql::Pool::new(url) {
        Ok(p) => p,
        Err(_e) => {
            println!("Could not connect! Did you set the username/password correctly?");
            exit(1);
        }
    };
    pool
}
