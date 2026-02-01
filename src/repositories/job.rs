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
