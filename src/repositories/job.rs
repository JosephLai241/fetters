//! Contains the job repository abstraction class.

use diesel::dsl::{count, sql};
use diesel::prelude::*;
use diesel::sql_types::Nullable;
use diesel::sqlite::Sqlite;
use diesel::{delete, insert_into, update};

use crate::cli::QueryArgs;
use crate::errors::FettersError;
use crate::models::insight::CountAndPercentage;
use crate::models::{
    job::{JobUpdate, NewJob, QueriedJob, TabledJob},
    sprint::QueriedSprint,
};
use crate::repositories::sprint::SprintRepository;
use crate::schema::{jobs, sprints, statuses, titles};

/// Contains all methods pertaining to CRUD operations for the `jobs` table.
pub struct JobRepository<'a> {
    pub connection: &'a mut SqliteConnection,
}

impl<'a> JobRepository<'a> {
    /// Adds a new job to the `jobs` table.
    pub fn add_job(&mut self, new_job: NewJob) -> Result<QueriedJob, FettersError> {
        use crate::schema::jobs::dsl::*;

        let queried_job = insert_into(jobs)
            .values(&new_job)
            .returning(QueriedJob::as_returning())
            .get_result(self.connection)?;

        let mut sprint_repo = SprintRepository {
            connection: self.connection,
        };
        sprint_repo.increment_num_jobs(new_job.sprint_id)?;

        Ok(queried_job)
    }

    /// Updates an existing job with new changes.
    pub fn update_job(
        &mut self,
        job_id: i32,
        changes: JobUpdate,
    ) -> Result<QueriedJob, FettersError> {
        use crate::schema::jobs::dsl::*;

        Ok(update(jobs.find(job_id))
            .set(&changes)
            .returning(QueriedJob::as_returning())
            .get_result(self.connection)?)
    }

    /// Deletes an existing job.
    pub fn delete_job(&mut self, job_id: i32) -> Result<QueriedJob, FettersError> {
        use crate::schema::jobs::dsl::*;

        let queried_job = delete(jobs.find(job_id))
            .returning(QueriedJob::as_returning())
            .get_result(self.connection)?;

        let mut sprint_repo = SprintRepository {
            connection: self.connection,
        };
        sprint_repo.decrement_num_jobs(queried_job.sprint_id)?;

        Ok(queried_job)
    }

    /// List all jobs matching the query.
    pub fn list_jobs(
        &mut self,
        query_args: &QueryArgs,
        current_sprint: &QueriedSprint,
    ) -> Result<Vec<TabledJob>, FettersError> {
        let mut query = jobs::table
            .left_join(titles::table.on(jobs::title_id.eq(titles::id)))
            .left_join(statuses::table.on(jobs::status_id.eq(statuses::id)))
            .left_join(sprints::table.on(jobs::sprint_id.eq(sprints::id)))
            .select((
                jobs::id,
                jobs::created,
                jobs::company_name,
                titles::name.nullable(),
                statuses::name.nullable(),
                sql::<Nullable<diesel::sql_types::Integer>>(
                    "NULLIF((SELECT COUNT(*) FROM interview_stages WHERE interview_stages.job_id = jobs.id), 0)",
                ),
                jobs::link,
                jobs::notes,
            ))
            .into_boxed::<Sqlite>();

        if let Some(sprint) = &query_args.sprint {
            query = query.filter(sprints::name.like(format!("%{}%", sprint)));
        } else {
            query = query.filter(sprints::id.eq(current_sprint.id));
        }

        if let Some(company) = &query_args.company {
            query = query.filter(jobs::company_name.like(format!("%{}%", company)));
        }

        if let Some(link) = &query_args.link {
            query = query.filter(jobs::link.like(format!("%{}%", link)));
        }

        if let Some(notes) = &query_args.notes {
            query = query.filter(jobs::notes.like(format!("%{}%", notes)));
        }

        if let Some(status) = &query_args.status {
            query = query.filter(statuses::name.like(format!("%{}%", status)));
        }

        if let Some(title) = &query_args.title {
            query = query.filter(titles::name.like(format!("%{}%", title)));
        }

        let mut jobs = query.load::<TabledJob>(self.connection)?;

        if let Some(stages_filter) = query_args.stages {
            if stages_filter == 0 {
                jobs.retain(|j| j.stages.is_some());
            } else {
                jobs.retain(|j| j.stages == Some(stages_filter));
            }
        }

        Ok(jobs)
    }

    /// Get the total number of jobs in the database.
    fn count_total_jobs(&mut self) -> Result<i64, FettersError> {
        use crate::schema::jobs::dsl::*;

        Ok(jobs.select(count(id)).first(self.connection)?)
    }

    /// Get the total number of jobs in the database by sprint.
    fn count_total_jobs_by_sprint(
        &mut self,
        current_sprint: &QueriedSprint,
    ) -> Result<i64, FettersError> {
        use crate::schema::jobs;

        Ok(jobs::table
            .left_join(sprints::table.on(jobs::sprint_id.eq(current_sprint.id)))
            .select(count(jobs::id))
            .first(self.connection)?)
    }

    /// Get the number of job applications and percentages per status for a given sprint.
    pub fn count_jobs_per_status(
        &mut self,
        current_sprint: &QueriedSprint,
    ) -> Result<Vec<CountAndPercentage>, FettersError> {
        use crate::schema::{jobs, statuses};

        let total_jobs = self.count_total_jobs()?;
        let total_jobs_in_sprint = self.count_total_jobs_by_sprint(current_sprint)?;

        let job_counts = jobs::table
            .left_join(statuses::table.on(jobs::status_id.eq(statuses::id)))
            .left_join(sprints::table.on(jobs::sprint_id.eq(sprints::id)))
            .group_by(statuses::name)
            .select((statuses::name.nullable(), count(jobs::id)))
            .filter(sprints::id.eq(current_sprint.id))
            .load::<(Option<String>, i64)>(self.connection)?;

        let mut jobs_per_status: Vec<CountAndPercentage> = Vec::new();
        for (status_name, count) in job_counts {
            if let Some(status) = status_name {
                jobs_per_status.push(CountAndPercentage {
                    label: status,
                    count,
                    sprint_percentage: format!(
                        "{:.2}%",
                        (count as f64 / total_jobs_in_sprint as f64) * 100.0
                    ),
                    overall_percentage: format!(
                        "{:.2}%",
                        (count as f64 / total_jobs as f64) * 100.0
                    ),
                });
            }
        }

        Ok(jobs_per_status)
    }

    /// Get the number of job applications and percentages for a given sprint.
    pub fn count_jobs_per_sprint(
        &mut self,
        current_sprint: &QueriedSprint,
    ) -> Result<Vec<CountAndPercentage>, FettersError> {
        use crate::schema::{jobs, sprints};

        let total_jobs = self.count_total_jobs()?;
        let total_jobs_in_sprint = self.count_total_jobs_by_sprint(current_sprint)?;

        let counts = jobs::table
            .left_join(sprints::table.on(jobs::sprint_id.eq(sprints::id)))
            .group_by(sprints::name)
            .select((sprints::name.nullable(), count(jobs::id)))
            .load::<(Option<String>, i64)>(self.connection)?;

        let mut jobs_per_sprint: Vec<CountAndPercentage> = Vec::new();
        for (sprint_name, count) in counts {
            if let Some(sprint_name) = sprint_name {
                jobs_per_sprint.push(CountAndPercentage {
                    label: sprint_name,
                    count,
                    sprint_percentage: format!(
                        "{:.2}%",
                        (count as f64 / total_jobs_in_sprint as f64) * 100.0
                    ),
                    overall_percentage: format!(
                        "{:.2}%",
                        (count as f64 / total_jobs as f64) * 100.0
                    ),
                });
            }
        }

        Ok(jobs_per_sprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::Connection;

    use crate::models::sprint::NewSprint;
    use crate::models::title::NewTitle;
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
        status_repo.seed_statuses().expect("Failed to seed statuses");

        connection
    }

    fn create_sprint(conn: &mut SqliteConnection, name: &str) -> QueriedSprint {
        let mut repo = SprintRepository { connection: conn };
        repo.add_job_sprint(NewSprint {
            name,
            start_date: "2025-01-01",
            end_date: None,
            num_jobs: &0,
        })
        .unwrap()
    }

    fn create_title(conn: &mut SqliteConnection, name: &str) -> crate::models::title::QueriedTitle {
        let mut repo = TitleRepository { connection: conn };
        repo.add_title(NewTitle { name }).unwrap()
    }

    fn get_status_id(conn: &mut SqliteConnection, target: &str) -> i32 {
        let mut repo = StatusRepository { connection: conn };
        let statuses = repo.get_all_statuses().unwrap();
        statuses.into_iter().find(|s| s.name == target).unwrap().id
    }

    #[test]
    fn test_add_job() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        let job = repo
            .add_job(NewJob {
                company_name: "Google",
                created: "2025-01-15 10:00:00".to_string(),
                title_id: title.id,
                status_id,
                link: Some("https://google.com/careers"),
                notes: Some("Dream job"),
                sprint_id: sprint.id,
            })
            .unwrap();

        assert_eq!(job.company_name, "Google");
        assert_eq!(job.title_id, title.id);
        assert_eq!(job.status_id, status_id);
        assert_eq!(job.link.as_deref(), Some("https://google.com/careers"));
        assert_eq!(job.notes.as_deref(), Some("Dream job"));
        assert_eq!(job.sprint_id, sprint.id);
    }

    #[test]
    fn test_add_job_increments_sprint_count() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        repo.add_job(NewJob {
            company_name: "Google",
            created: "2025-01-15 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();

        let mut sprint_repo = SprintRepository {
            connection: &mut conn,
        };
        let updated_sprint = sprint_repo.get_current_sprint("test-sprint").unwrap();
        assert_eq!(updated_sprint.num_jobs, 1);
    }

    #[test]
    fn test_update_job() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        let job = repo
            .add_job(NewJob {
                company_name: "Google",
                created: "2025-01-15 10:00:00".to_string(),
                title_id: title.id,
                status_id,
                link: None,
                notes: None,
                sprint_id: sprint.id,
            })
            .unwrap();

        let updated = repo
            .update_job(
                job.id,
                JobUpdate {
                    company_name: Some("Alphabet"),
                    notes: Some("Updated notes"),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(updated.company_name, "Alphabet");
        assert_eq!(updated.notes.as_deref(), Some("Updated notes"));
    }

    #[test]
    fn test_delete_job() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        let job = repo
            .add_job(NewJob {
                company_name: "Google",
                created: "2025-01-15 10:00:00".to_string(),
                title_id: title.id,
                status_id,
                link: None,
                notes: None,
                sprint_id: sprint.id,
            })
            .unwrap();

        let deleted = repo.delete_job(job.id).unwrap();
        assert_eq!(deleted.id, job.id);
        assert_eq!(deleted.company_name, "Google");
    }

    #[test]
    fn test_delete_job_decrements_sprint_count() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        let job = repo
            .add_job(NewJob {
                company_name: "Google",
                created: "2025-01-15 10:00:00".to_string(),
                title_id: title.id,
                status_id,
                link: None,
                notes: None,
                sprint_id: sprint.id,
            })
            .unwrap();

        repo.delete_job(job.id).unwrap();

        let mut sprint_repo = SprintRepository {
            connection: &mut conn,
        };
        let updated_sprint = sprint_repo.get_current_sprint("test-sprint").unwrap();
        assert_eq!(updated_sprint.num_jobs, 0);
    }

    #[test]
    fn test_list_jobs_returns_jobs_in_sprint() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        repo.add_job(NewJob {
            company_name: "Google",
            created: "2025-01-15 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();
        repo.add_job(NewJob {
            company_name: "Meta",
            created: "2025-01-16 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();

        let query_args = QueryArgs::default();
        let jobs = repo.list_jobs(&query_args, &sprint).unwrap();
        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn test_list_jobs_filters_by_company() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        repo.add_job(NewJob {
            company_name: "Google",
            created: "2025-01-15 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();
        repo.add_job(NewJob {
            company_name: "Meta",
            created: "2025-01-16 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();

        let query_args = QueryArgs {
            company: Some("Goo".to_string()),
            ..Default::default()
        };
        let jobs = repo.list_jobs(&query_args, &sprint).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].company_name, "Google");
    }

    #[test]
    fn test_list_jobs_filters_by_status() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let pending_id = get_status_id(&mut conn, "PENDING");
        let rejected_id = get_status_id(&mut conn, "REJECTED");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        repo.add_job(NewJob {
            company_name: "Google",
            created: "2025-01-15 10:00:00".to_string(),
            title_id: title.id,
            status_id: pending_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();
        repo.add_job(NewJob {
            company_name: "Meta",
            created: "2025-01-16 10:00:00".to_string(),
            title_id: title.id,
            status_id: rejected_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();

        let query_args = QueryArgs {
            status: Some("REJECTED".to_string()),
            ..Default::default()
        };
        let jobs = repo.list_jobs(&query_args, &sprint).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].company_name, "Meta");
    }

    #[test]
    fn test_list_jobs_empty_when_no_match() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        repo.add_job(NewJob {
            company_name: "Google",
            created: "2025-01-15 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();

        let query_args = QueryArgs {
            company: Some("Nonexistent".to_string()),
            ..Default::default()
        };
        let jobs = repo.list_jobs(&query_args, &sprint).unwrap();
        assert_eq!(jobs.len(), 0);
    }

    #[test]
    fn test_list_jobs_across_sprints() {
        let mut conn = setup_test_db();
        let sprint1 = create_sprint(&mut conn, "sprint-1");
        let sprint2 = create_sprint(&mut conn, "sprint-2");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        repo.add_job(NewJob {
            company_name: "Google",
            created: "2025-01-15 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint1.id,
        })
        .unwrap();
        repo.add_job(NewJob {
            company_name: "Meta",
            created: "2025-01-16 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint2.id,
        })
        .unwrap();

        // Searching by sprint name should find across sprints
        let query_args = QueryArgs {
            sprint: Some("sprint".to_string()),
            ..Default::default()
        };
        let jobs = repo.list_jobs(&query_args, &sprint1).unwrap();
        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn test_count_jobs_per_status() {
        let mut conn = setup_test_db();
        let sprint = create_sprint(&mut conn, "test-sprint");
        let title = create_title(&mut conn, "SWE");
        let pending_id = get_status_id(&mut conn, "PENDING");
        let rejected_id = get_status_id(&mut conn, "REJECTED");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        repo.add_job(NewJob {
            company_name: "Google",
            created: "2025-01-15 10:00:00".to_string(),
            title_id: title.id,
            status_id: pending_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();
        repo.add_job(NewJob {
            company_name: "Meta",
            created: "2025-01-16 10:00:00".to_string(),
            title_id: title.id,
            status_id: rejected_id,
            link: None,
            notes: None,
            sprint_id: sprint.id,
        })
        .unwrap();

        let insights = repo.count_jobs_per_status(&sprint).unwrap();
        assert_eq!(insights.len(), 2);

        let pending = insights.iter().find(|i| i.label == "PENDING").unwrap();
        assert_eq!(pending.count, 1);

        let rejected = insights.iter().find(|i| i.label == "REJECTED").unwrap();
        assert_eq!(rejected.count, 1);
    }

    #[test]
    fn test_count_jobs_per_sprint() {
        let mut conn = setup_test_db();
        let sprint1 = create_sprint(&mut conn, "sprint-1");
        let sprint2 = create_sprint(&mut conn, "sprint-2");
        let title = create_title(&mut conn, "SWE");
        let status_id = get_status_id(&mut conn, "PENDING");

        let mut repo = JobRepository {
            connection: &mut conn,
        };
        repo.add_job(NewJob {
            company_name: "Google",
            created: "2025-01-15 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint1.id,
        })
        .unwrap();
        repo.add_job(NewJob {
            company_name: "Meta",
            created: "2025-01-16 10:00:00".to_string(),
            title_id: title.id,
            status_id,
            link: None,
            notes: None,
            sprint_id: sprint2.id,
        })
        .unwrap();

        let insights = repo.count_jobs_per_sprint(&sprint1).unwrap();
        assert!(insights.len() >= 2);

        let s1 = insights.iter().find(|i| i.label == "sprint-1").unwrap();
        assert_eq!(s1.count, 1);

        let s2 = insights.iter().find(|i| i.label == "sprint-2").unwrap();
        assert_eq!(s2.count, 1);
    }
}
