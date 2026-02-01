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
