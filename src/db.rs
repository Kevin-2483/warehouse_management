use diesel::sqlite::SqliteConnection;
use diesel::Connection;

pub fn establish_connection() -> Result<SqliteConnection, diesel::ConnectionError> {
    let database_url = "sqlite://./warehouse.db";
    SqliteConnection::establish(&database_url)
        .map_err(|e| e.into())
} 