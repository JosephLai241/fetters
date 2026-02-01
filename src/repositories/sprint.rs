//! Contains the job sprint repository abstraction class.

use chrono::Local;
use diesel::dsl::update;
use diesel::insert_into;
use diesel::prelude::*;

use crate::errors::FettersError;
use crate::models::sprint::{NewSprint, QueriedSprint, SprintUpdate};
use crate::schema::sprints;

/// Contains all methods pertaining to CRUD operations for the `sprints` table.
pub struct SprintRepository<'a> {
    pub connection: &'a mut SqliteConnection,
}

impl<'a> SprintRepository<'a> {
    /// Adds a new job sprint into the `sprints` table.
    pub fn add_job_sprint(&mut self, new_sprint: NewSprint) -> Result<QueriedSprint, FettersError> {
        use crate::schema::sprints::dsl::*;

        Ok(insert_into(sprints)
            .values(&new_sprint)
            .returning(QueriedSprint::as_returning())
            .get_result(self.connection)?)
    }

    /// Retrieves the current sprint's ID.
    pub fn get_current_sprint(&mut self, sprint_name: &str) -> Result<QueriedSprint, FettersError> {
        use crate::schema::sprints::dsl::*;

        sprints
            .filter(name.eq(sprint_name))
            .select(QueriedSprint::as_select())
            .first::<QueriedSprint>(self.connection)
            .optional()?
            .map_or_else(
                || {
                    let new_sprint = NewSprint {
                        name: sprint_name,
                        start_date: &Local::now().date_naive().format("%Y-%m-%d").to_string(),
                        end_date: None,
                        num_jobs: &0,
                    };
                    self.add_job_sprint(new_sprint)
                },
                Ok,
            )
    }

    /// Update an existing sprint with new changes.
    pub fn update_sprint(
        &mut self,
        sprint_id: i32,
        changes: SprintUpdate,
    ) -> Result<QueriedSprint, FettersError> {
        use crate::schema::sprints::dsl::*;

        Ok(update(sprints.find(sprint_id))
            .set(&changes)
            .returning(QueriedSprint::as_returning())
            .get_result(self.connection)?)
    }

    /// Retrieves all job sprints.
    pub fn get_all_sprints(&mut self) -> Result<Vec<QueriedSprint>, FettersError> {
        use crate::schema::sprints::dsl::*;

        Ok(sprints
            .select(QueriedSprint::as_select())
            .load(self.connection)?)
    }

    /// Increment the `num_jobs` count for a particular sprint.
    pub fn increment_num_jobs(&mut self, sprint_id: i32) -> Result<(), FettersError> {
        update(sprints::table.filter(sprints::id.eq(sprint_id)))
            .set(sprints::num_jobs.eq(sprints::num_jobs + 1))
            .execute(self.connection)?;

        Ok(())
    }

    /// Decrement the `num_jobs` count for a particular sprint.
    pub fn decrement_num_jobs(&mut self, sprint_id: i32) -> Result<(), FettersError> {
        update(sprints::table.filter(sprints::id.eq(sprint_id)))
            .set(sprints::num_jobs.eq(sprints::num_jobs - 1))
            .execute(self.connection)?;

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
    fn test_add_job_sprint() {
        let mut conn = setup_test_db();
        let mut repo = SprintRepository {
            connection: &mut conn,
        };

        let sprint = NewSprint {
            name: "test-sprint",
            start_date: "2025-01-15",
            end_date: None,
            num_jobs: &0,
        };
        let result = repo.add_job_sprint(sprint).unwrap();
        assert_eq!(result.name, "test-sprint");
        assert_eq!(result.start_date, "2025-01-15");
        assert_eq!(result.end_date, None);
        assert_eq!(result.num_jobs, 0);
    }

    #[test]
    fn test_get_current_sprint_creates_if_missing() {
        let mut conn = setup_test_db();
        let mut repo = SprintRepository {
            connection: &mut conn,
        };

        let sprint = repo.get_current_sprint("new-sprint").unwrap();
        assert_eq!(sprint.name, "new-sprint");
        assert_eq!(sprint.num_jobs, 0);
    }

    #[test]
    fn test_get_current_sprint_returns_existing() {
        let mut conn = setup_test_db();
        let mut repo = SprintRepository {
            connection: &mut conn,
        };

        let new_sprint = NewSprint {
            name: "existing-sprint",
            start_date: "2025-01-01",
            end_date: None,
            num_jobs: &5,
        };
        repo.add_job_sprint(new_sprint).unwrap();

        let sprint = repo.get_current_sprint("existing-sprint").unwrap();
        assert_eq!(sprint.name, "existing-sprint");
        assert_eq!(sprint.num_jobs, 5);
    }

    #[test]
    fn test_update_sprint() {
        let mut conn = setup_test_db();
        let mut repo = SprintRepository {
            connection: &mut conn,
        };

        let sprint = repo
            .add_job_sprint(NewSprint {
                name: "sprint-1",
                start_date: "2025-01-01",
                end_date: None,
                num_jobs: &0,
            })
            .unwrap();

        let changes = SprintUpdate {
            name: Some("sprint-renamed"),
            end_date: Some(Some("2025-02-01")),
            ..Default::default()
        };
        let updated = repo.update_sprint(sprint.id, changes).unwrap();
        assert_eq!(updated.name, "sprint-renamed");
        assert_eq!(updated.end_date, Some("2025-02-01".to_string()));
    }

    #[test]
    fn test_get_all_sprints() {
        let mut conn = setup_test_db();
        let mut repo = SprintRepository {
            connection: &mut conn,
        };

        repo.add_job_sprint(NewSprint {
            name: "sprint-1",
            start_date: "2025-01-01",
            end_date: None,
            num_jobs: &0,
        })
        .unwrap();
        repo.add_job_sprint(NewSprint {
            name: "sprint-2",
            start_date: "2025-02-01",
            end_date: None,
            num_jobs: &3,
        })
        .unwrap();

        let sprints = repo.get_all_sprints().unwrap();
        assert_eq!(sprints.len(), 2);
    }

    #[test]
    fn test_get_all_sprints_empty() {
        let mut conn = setup_test_db();
        let mut repo = SprintRepository {
            connection: &mut conn,
        };

        let sprints = repo.get_all_sprints().unwrap();
        assert_eq!(sprints.len(), 0);
    }

    #[test]
    fn test_increment_num_jobs() {
        let mut conn = setup_test_db();
        let mut repo = SprintRepository {
            connection: &mut conn,
        };

        let sprint = repo
            .add_job_sprint(NewSprint {
                name: "sprint-inc",
                start_date: "2025-01-01",
                end_date: None,
                num_jobs: &0,
            })
            .unwrap();
        assert_eq!(sprint.num_jobs, 0);

        repo.increment_num_jobs(sprint.id).unwrap();

        let updated = repo.get_current_sprint("sprint-inc").unwrap();
        assert_eq!(updated.num_jobs, 1);
    }

    #[test]
    fn test_decrement_num_jobs() {
        let mut conn = setup_test_db();
        let mut repo = SprintRepository {
            connection: &mut conn,
        };

        let sprint = repo
            .add_job_sprint(NewSprint {
                name: "sprint-dec",
                start_date: "2025-01-01",
                end_date: None,
                num_jobs: &3,
            })
            .unwrap();

        repo.decrement_num_jobs(sprint.id).unwrap();

        let updated = repo.get_current_sprint("sprint-dec").unwrap();
        assert_eq!(updated.num_jobs, 2);
    }
}
