use anyhow::Result;
use rusqlite::Connection;

pub fn conn(database_url: &str) -> Result<Connection> {
    let conn = Connection::open(database_url)?;
    
    // Initialize the database tables
    crate::database::create_database_with_connection(&conn)?;
    
    Ok(conn)
}