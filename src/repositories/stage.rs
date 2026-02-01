//! Contains the interview stage repository abstraction class.

use diesel::dsl::max;
use diesel::prelude::*;
use diesel::{delete, insert_into, update};

use crate::errors::FettersError;
use crate::models::stage::{InterviewStageUpdate, NewInterviewStage, QueriedInterviewStage};
use crate::schema::interview_stages;

/// Contains all methods pertaining to CRUD operations for the `interview_stages` table.
pub struct StageRepository<'a> {
    /// A mutable reference to the SQLite database connection.
    pub connection: &'a mut SqliteConnection,
}

impl<'a> StageRepository<'a> {
    /// Adds a new interview stage.
    pub fn add_stage(
        &mut self,
        new_stage: NewInterviewStage,
    ) -> Result<QueriedInterviewStage, FettersError> {
        Ok(insert_into(interview_stages::table)
            .values(&new_stage)
            .returning(QueriedInterviewStage::as_returning())
            .get_result(self.connection)?)
    }

    /// Gets all interview stages for a given job, ordered by stage number.
    pub fn get_stages_for_job(
        &mut self,
        target_job_id: i32,
    ) -> Result<Vec<QueriedInterviewStage>, FettersError> {
        Ok(interview_stages::table
            .filter(interview_stages::job_id.eq(target_job_id))
            .order(interview_stages::stage_number.asc())
            .select(QueriedInterviewStage::as_select())
            .load(self.connection)?)
    }

    /// Gets the next stage number for a given job (MAX + 1, or 1 if none exist).
    pub fn get_next_stage_number(&mut self, target_job_id: i32) -> Result<i32, FettersError> {
        let max_stage: Option<i32> = interview_stages::table
            .filter(interview_stages::job_id.eq(target_job_id))
            .select(max(interview_stages::stage_number))
            .first(self.connection)?;

        Ok(max_stage.unwrap_or(0) + 1)
    }

    /// Updates an existing interview stage.
    pub fn update_stage(
        &mut self,
        stage_id: i32,
        changes: InterviewStageUpdate,
    ) -> Result<QueriedInterviewStage, FettersError> {
        Ok(
            update(interview_stages::table.find(stage_id))
                .set(&changes)
                .returning(QueriedInterviewStage::as_returning())
                .get_result(self.connection)?,
        )
    }

    /// Deletes an interview stage.
    pub fn delete_stage(
        &mut self,
        stage_id: i32,
    ) -> Result<QueriedInterviewStage, FettersError> {
        Ok(
            delete(interview_stages::table.find(stage_id))
                .returning(QueriedInterviewStage::as_returning())
                .get_result(self.connection)?,
        )
    }

    /// Renumber stages for a given job after deletion so they are sequential (1, 2, 3...).
    pub fn renumber_stages(&mut self, target_job_id: i32) -> Result<(), FettersError> {
        let stages = self.get_stages_for_job(target_job_id)?;

        for (index, stage) in stages.iter().enumerate() {
            let new_number = (index + 1) as i32;
            if stage.stage_number != new_number {
                update(interview_stages::table.find(stage.id))
                    .set(interview_stages::stage_number.eq(new_number))
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

    use crate::models::job::NewJob;
    use crate::models::sprint::NewSprint;
    use crate::models::title::NewTitle;
    use crate::repositories::job::JobRepository;
    use crate::repositories::sprint::SprintRepository;
    use crate::repositories::statuses::StatusRepository;
    use crate::repositories::title::TitleRepository;

    fn setup_test_db() -> SqliteConnection {
        let mut connection = SqliteConnection::establish(":memory:")
            .expect("Failed to create in-memory database");
        crate::utils::migrations::run_migrations(&mut connection)
            .expect("Failed to run migrations");

        let mut status_repo = StatusRepository {
            connection: &mut connection,
        };
        status_repo
            .seed_statuses()
            .expect("Failed to seed statuses");

        connection
    }

    fn create_test_job(conn: &mut SqliteConnection) -> crate::models::job::QueriedJob {
        let mut sprint_repo = SprintRepository { connection: conn };
        let sprint = sprint_repo
            .add_job_sprint(NewSprint {
                name: "test-sprint",
                start_date: "2025-01-01",
                end_date: None,
                num_jobs: &0,
            })
            .unwrap();

        let mut title_repo = TitleRepository { connection: conn };
        let title = title_repo.add_title(NewTitle { name: "SWE" }).unwrap();

        let mut status_repo = StatusRepository { connection: conn };
        let statuses = status_repo.get_all_statuses().unwrap();
        let status_id = statuses[0].id;

        let mut job_repo = JobRepository { connection: conn };
        job_repo
            .add_job(NewJob {
                company_name: "TestCo",
                created: "2025-01-15 10:00:00".to_string(),
                title_id: title.id,
                status_id,
                link: None,
                notes: None,
                sprint_id: sprint.id,
            })
            .unwrap()
    }

    #[test]
    fn test_add_stage() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        let stage = repo
            .add_stage(NewInterviewStage {
                job_id: job.id,
                stage_number: 1,
                name: Some("Phone Screen".to_string()),
                status: "SCHEDULED".to_string(),
                scheduled_date: "2025/01/20".to_string(),
                notes: Some("Prep for this".to_string()),
                created: "2025-01-15".to_string(),
            })
            .unwrap();

        assert_eq!(stage.job_id, job.id);
        assert_eq!(stage.stage_number, 1);
        assert_eq!(stage.name.as_deref(), Some("Phone Screen"));
        assert_eq!(stage.status, "SCHEDULED");
        assert_eq!(stage.scheduled_date, "2025/01/20");
        assert_eq!(stage.notes.as_deref(), Some("Prep for this"));
    }

    #[test]
    fn test_get_stages_for_job() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        repo.add_stage(NewInterviewStage {
            job_id: job.id,
            stage_number: 1,
            name: Some("Phone".to_string()),
            status: "PASSED".to_string(),
            scheduled_date: "2025/01/20".to_string(),
            notes: None,
            created: "2025-01-15".to_string(),
        })
        .unwrap();
        repo.add_stage(NewInterviewStage {
            job_id: job.id,
            stage_number: 2,
            name: Some("Onsite".to_string()),
            status: "SCHEDULED".to_string(),
            scheduled_date: "2025/02/01".to_string(),
            notes: None,
            created: "2025-01-20".to_string(),
        })
        .unwrap();

        let stages = repo.get_stages_for_job(job.id).unwrap();
        assert_eq!(stages.len(), 2);
        assert_eq!(stages[0].stage_number, 1);
        assert_eq!(stages[1].stage_number, 2);
    }

    #[test]
    fn test_get_stages_for_job_empty() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        let stages = repo.get_stages_for_job(job.id).unwrap();
        assert_eq!(stages.len(), 0);
    }

    #[test]
    fn test_get_next_stage_number_first() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        let next = repo.get_next_stage_number(job.id).unwrap();
        assert_eq!(next, 1);
    }

    #[test]
    fn test_get_next_stage_number_after_existing() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        repo.add_stage(NewInterviewStage {
            job_id: job.id,
            stage_number: 1,
            name: None,
            status: "SCHEDULED".to_string(),
            scheduled_date: "2025/01/20".to_string(),
            notes: None,
            created: "2025-01-15".to_string(),
        })
        .unwrap();
        repo.add_stage(NewInterviewStage {
            job_id: job.id,
            stage_number: 2,
            name: None,
            status: "SCHEDULED".to_string(),
            scheduled_date: "2025/02/01".to_string(),
            notes: None,
            created: "2025-01-20".to_string(),
        })
        .unwrap();

        let next = repo.get_next_stage_number(job.id).unwrap();
        assert_eq!(next, 3);
    }

    #[test]
    fn test_update_stage() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        let stage = repo
            .add_stage(NewInterviewStage {
                job_id: job.id,
                stage_number: 1,
                name: Some("Phone".to_string()),
                status: "SCHEDULED".to_string(),
                scheduled_date: "2025/01/20".to_string(),
                notes: None,
                created: "2025-01-15".to_string(),
            })
            .unwrap();

        let updated = repo
            .update_stage(
                stage.id,
                InterviewStageUpdate {
                    status: Some("PASSED".to_string()),
                    notes: Some("Went great".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.status, "PASSED");
        assert_eq!(updated.notes.as_deref(), Some("Went great"));
        assert_eq!(updated.name.as_deref(), Some("Phone"));
    }

    #[test]
    fn test_delete_stage() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        let stage = repo
            .add_stage(NewInterviewStage {
                job_id: job.id,
                stage_number: 1,
                name: None,
                status: "SCHEDULED".to_string(),
                scheduled_date: "2025/01/20".to_string(),
                notes: None,
                created: "2025-01-15".to_string(),
            })
            .unwrap();

        let deleted = repo.delete_stage(stage.id).unwrap();
        assert_eq!(deleted.id, stage.id);

        let stages = repo.get_stages_for_job(job.id).unwrap();
        assert_eq!(stages.len(), 0);
    }

    #[test]
    fn test_renumber_stages_after_deletion() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        repo.add_stage(NewInterviewStage {
            job_id: job.id,
            stage_number: 1,
            name: Some("Phone".to_string()),
            status: "PASSED".to_string(),
            scheduled_date: "2025/01/20".to_string(),
            notes: None,
            created: "2025-01-15".to_string(),
        })
        .unwrap();
        let stage2 = repo
            .add_stage(NewInterviewStage {
                job_id: job.id,
                stage_number: 2,
                name: Some("Onsite".to_string()),
                status: "SCHEDULED".to_string(),
                scheduled_date: "2025/02/01".to_string(),
                notes: None,
                created: "2025-01-20".to_string(),
            })
            .unwrap();
        repo.add_stage(NewInterviewStage {
            job_id: job.id,
            stage_number: 3,
            name: Some("Final".to_string()),
            status: "SCHEDULED".to_string(),
            scheduled_date: "2025/02/15".to_string(),
            notes: None,
            created: "2025-01-25".to_string(),
        })
        .unwrap();

        // Delete stage 2 (the middle one)
        repo.delete_stage(stage2.id).unwrap();
        repo.renumber_stages(job.id).unwrap();

        let stages = repo.get_stages_for_job(job.id).unwrap();
        assert_eq!(stages.len(), 2);
        assert_eq!(stages[0].stage_number, 1);
        assert_eq!(stages[0].name.as_deref(), Some("Phone"));
        assert_eq!(stages[1].stage_number, 2);
        assert_eq!(stages[1].name.as_deref(), Some("Final"));
    }

    #[test]
    fn test_renumber_stages_no_op_when_sequential() {
        let mut conn = setup_test_db();
        let job = create_test_job(&mut conn);

        let mut repo = StageRepository {
            connection: &mut conn,
        };
        repo.add_stage(NewInterviewStage {
            job_id: job.id,
            stage_number: 1,
            name: Some("Phone".to_string()),
            status: "PASSED".to_string(),
            scheduled_date: "2025/01/20".to_string(),
            notes: None,
            created: "2025-01-15".to_string(),
        })
        .unwrap();
        repo.add_stage(NewInterviewStage {
            job_id: job.id,
            stage_number: 2,
            name: Some("Onsite".to_string()),
            status: "SCHEDULED".to_string(),
            scheduled_date: "2025/02/01".to_string(),
            notes: None,
            created: "2025-01-20".to_string(),
        })
        .unwrap();

        // Renumber when already sequential should be a no-op
        repo.renumber_stages(job.id).unwrap();

        let stages = repo.get_stages_for_job(job.id).unwrap();
        assert_eq!(stages[0].stage_number, 1);
        assert_eq!(stages[1].stage_number, 2);
    }
}
