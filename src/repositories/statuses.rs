//! Contains the statuses repository abstraction class.

use diesel::insert_into;
use diesel::prelude::*;
use lazy_static::lazy_static;

use crate::errors::FettersError;
use crate::models::status::{NewStatus, QueriedStatus};

lazy_static! {
    /// Contains all default statuses that will be stored into the `statuses` SQLite table on the
    /// initial run.
    static ref DEFAULT_STATUSES: Vec<&'static str> = vec![
        "GHOSTED",
        "HIRED",
        "IN PROGRESS",
        "NOT HIRING ANYMORE",
        "OFFER RECEIVED",
        "PENDING",
        "REJECTED",
    ];
}

/// Contains all methods pertaining to CRUD operations for the `statuses` table.
pub struct StatusRepository<'a> {
    pub connection: &'a mut SqliteConnection,
}

impl<'a> StatusRepository<'a> {
    /// Retrieves all statuses.
    pub fn get_all_statuses(&mut self) -> Result<Vec<QueriedStatus>, FettersError> {
        use crate::schema::statuses::dsl::*;

        Ok(statuses
            .select(QueriedStatus::as_select())
            .load(self.connection)?)
    }

    /// Stores the default statuses into the `statuses` table if it doesn't already exist.
    pub fn seed_statuses(&mut self) -> Result<(), FettersError> {
        use crate::schema::statuses::dsl::*;

        for status in DEFAULT_STATUSES.iter().copied() {
            let exists = statuses
                .filter(name.eq(status))
                .select(QueriedStatus::as_select())
                .first(self.connection)
                .optional()?;

            if exists.is_none() {
                let new_status = NewStatus { name: status };
                insert_into(statuses)
                    .values(&new_status)
                    .execute(self.connection)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::Connection;

    fn setup_test_db() -> SqliteConnection {
        let mut connection = SqliteConnection::establish(":memory:")
            .expect("Failed to create in-memory database");
        crate::utils::migrations::run_migrations(&mut connection)
            .expect("Failed to run migrations");
        connection
    }

    #[test]
    fn test_get_all_statuses_empty_initially() {
        let mut connection = setup_test_db();
        let mut repo = StatusRepository {
            connection: &mut connection,
        };

        let statuses = repo.get_all_statuses().unwrap();
        assert_eq!(statuses.len(), 0);
    }

    #[test]
    fn test_seed_statuses_creates_default_statuses() {
        let mut connection = setup_test_db();
        let mut repo = StatusRepository {
            connection: &mut connection,
        };

        repo.seed_statuses().unwrap();

        let statuses = repo.get_all_statuses().unwrap();
        assert_eq!(statuses.len(), 7);

        let names: Vec<String> = statuses.into_iter().map(|s| s.name).collect();
        assert!(names.contains(&"GHOSTED".to_string()));
        assert!(names.contains(&"HIRED".to_string()));
        assert!(names.contains(&"IN PROGRESS".to_string()));
        assert!(names.contains(&"NOT HIRING ANYMORE".to_string()));
        assert!(names.contains(&"OFFER RECEIVED".to_string()));
        assert!(names.contains(&"PENDING".to_string()));
        assert!(names.contains(&"REJECTED".to_string()));
    }

    #[test]
    fn test_seed_statuses_is_idempotent() {
        let mut connection = setup_test_db();
        let mut repo = StatusRepository {
            connection: &mut connection,
        };

        repo.seed_statuses().unwrap();
        repo.seed_statuses().unwrap();

        let statuses = repo.get_all_statuses().unwrap();
        assert_eq!(statuses.len(), 7);
    }
}
