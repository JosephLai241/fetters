//! Contains all functionality pertaining to interacting with SQLite.

use diesel::Connection;
use diesel::sqlite::SqliteConnection;

use crate::errors::FettersError;

/// Contains all functionality pertaining to interacting with the SQLite database.
pub struct Database {
    /// The SQLite connection.
    pub connection: SqliteConnection,
}

impl Database {
    /// Create a new connection to the SQLite database.
    pub fn new_connection(db_path: &str) -> Result<Database, FettersError> {
        let connection = SqliteConnection::establish(db_path)?;
        Ok(Database { connection })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_connection_in_memory() {
        let db = Database::new_connection(":memory:");
        assert!(db.is_ok());
    }

    #[test]
    fn test_new_connection_invalid_path_fails() {
        let db = Database::new_connection("/nonexistent/path/to/database.db");
        assert!(db.is_err());
    }
}
