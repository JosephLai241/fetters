//! Contains all models for job statuses.

use std::fmt::{self, Display, Formatter};

use diesel::sqlite::Sqlite;
use diesel::{Insertable, Queryable, Selectable};

use crate::schema::statuses;

/// This struct defines a new status that will be written to the `sprints` table in SQLite.
#[derive(Debug, Insertable)]
#[diesel(table_name = statuses)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewStatus<'a> {
    /// The status name.
    pub name: &'a str,
}

/// This struct defines the status object returned from querying SQLite.
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = statuses)]
#[diesel(check_for_backend(Sqlite))]
pub struct QueriedStatus {
    /// The SQLite ID.
    pub id: i32,
    /// The status title.
    pub name: String,
}

/// Implementing `Display` allows this struct to be displayed in the `Select` Inquire menu.
impl Display for QueriedStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queried_status_display() {
        let status = QueriedStatus {
            id: 1,
            name: "PENDING".to_string(),
        };
        assert_eq!(format!("{}", status), "PENDING");
    }
}
