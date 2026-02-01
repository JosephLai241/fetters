//! Contains all models for job titles.

use std::fmt::{self, Display, Formatter};

use diesel::sqlite::Sqlite;
use diesel::{Insertable, Queryable, Selectable};

use crate::schema::titles;

/// This struct defines a new job title that will be written to the `titles` table in SQLite.
#[derive(Debug, Insertable)]
#[diesel(table_name = titles)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewTitle<'a> {
    /// The job title.
    pub name: &'a str,
}

/// This struct defines the title object returned from querying SQLite.
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = titles)]
#[diesel(check_for_backend(Sqlite))]
pub struct QueriedTitle {
    /// The SQLite ID.
    pub id: i32,
    /// The job title.
    pub name: String,
}

/// Implementing `Display` allows this struct to be displayed in the `Select` Inquire menu.
impl Display for QueriedTitle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queried_title_display() {
        let title = QueriedTitle {
            id: 1,
            name: "Software Engineer".to_string(),
        };
        assert_eq!(format!("{}", title), "Software Engineer");
    }
}
