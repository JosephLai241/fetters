//! Contains the title repository abstraction class.

use diesel::insert_into;
use diesel::prelude::*;

use crate::errors::FettersError;
use crate::models::title::{NewTitle, QueriedTitle};

/// Contains all methods pertaining to CRUD operations for the `titles` table.
pub struct TitleRepository<'a> {
    pub connection: &'a mut SqliteConnection,
}

impl<'a> TitleRepository<'a> {
    /// Adds a new job title into the `titles` table.
    pub fn add_title(&mut self, new_title: NewTitle) -> Result<QueriedTitle, FettersError> {
        use crate::schema::titles::dsl::*;

        insert_into(titles)
            .values(&new_title)
            .on_conflict(name)
            .do_nothing()
            .execute(self.connection)?;

        Ok(titles
            .filter(name.eq(new_title.name))
            .first(self.connection)?)
    }

    /// Retrieves an existing job title by ID.
    pub fn get_title(&mut self, title_id: i32) -> Result<QueriedTitle, FettersError> {
        use crate::schema::titles::dsl::*;

        Ok(titles
            .find(title_id)
            .select(QueriedTitle::as_select())
            .first(self.connection)?)
    }

    /// Retrieves all job titles.
    pub fn get_all_titles(&mut self) -> Result<Vec<QueriedTitle>, FettersError> {
        use crate::schema::titles::dsl::*;

        Ok(titles
            .select(QueriedTitle::as_select())
            .load(self.connection)?)
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
    fn test_add_title() {
        let mut conn = setup_test_db();
        let mut repo = TitleRepository {
            connection: &mut conn,
        };

        let title = repo
            .add_title(NewTitle {
                name: "Software Engineer",
            })
            .unwrap();
        assert_eq!(title.name, "Software Engineer");
        assert!(title.id > 0);
    }

    #[test]
    fn test_add_duplicate_title_returns_existing() {
        let mut conn = setup_test_db();
        let mut repo = TitleRepository {
            connection: &mut conn,
        };

        let first = repo.add_title(NewTitle { name: "SWE" }).unwrap();
        let second = repo.add_title(NewTitle { name: "SWE" }).unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(first.name, second.name);
    }

    #[test]
    fn test_get_title() {
        let mut conn = setup_test_db();
        let mut repo = TitleRepository {
            connection: &mut conn,
        };

        let added = repo
            .add_title(NewTitle {
                name: "Data Scientist",
            })
            .unwrap();
        let fetched = repo.get_title(added.id).unwrap();
        assert_eq!(fetched.id, added.id);
        assert_eq!(fetched.name, "Data Scientist");
    }

    #[test]
    fn test_get_title_not_found() {
        let mut conn = setup_test_db();
        let mut repo = TitleRepository {
            connection: &mut conn,
        };

        let result = repo.get_title(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_titles() {
        let mut conn = setup_test_db();
        let mut repo = TitleRepository {
            connection: &mut conn,
        };

        repo.add_title(NewTitle { name: "SWE" }).unwrap();
        repo.add_title(NewTitle { name: "PM" }).unwrap();
        repo.add_title(NewTitle { name: "Designer" }).unwrap();

        let titles = repo.get_all_titles().unwrap();
        assert_eq!(titles.len(), 3);
    }

    #[test]
    fn test_get_all_titles_empty() {
        let mut conn = setup_test_db();
        let mut repo = TitleRepository {
            connection: &mut conn,
        };

        let titles = repo.get_all_titles().unwrap();
        assert_eq!(titles.len(), 0);
    }
}
