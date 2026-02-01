//! Contains all models for interview stages.

use std::fmt::{self, Display, Formatter};

use diesel::sqlite::Sqlite;
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use owo_colors::OwoColorize;

use crate::schema::interview_stages;

/// The status of an interview stage.
#[derive(Clone, Debug)]
pub enum StageStatus {
    /// The interview is scheduled but has not yet occurred.
    Scheduled,
    /// The interview stage has been passed.
    Passed,
    /// The interview stage resulted in a rejection.
    Rejected,
}

impl StageStatus {
    /// Returns all variants for use in `inquire::Select` prompts.
    pub fn variants() -> Vec<StageStatus> {
        vec![
            StageStatus::Scheduled,
            StageStatus::Passed,
            StageStatus::Rejected,
        ]
    }

    /// Returns the string representation stored in SQLite.
    pub fn as_str(&self) -> &'static str {
        match self {
            StageStatus::Scheduled => "SCHEDULED",
            StageStatus::Passed => "PASSED",
            StageStatus::Rejected => "REJECTED",
        }
    }

    /// Returns a date prompt label appropriate for this status.
    pub fn date_prompt(&self) -> &'static str {
        match self {
            StageStatus::Scheduled => "Select the scheduled date:",
            StageStatus::Passed => "Select the passed date:",
            StageStatus::Rejected => "Select the rejected date:",
        }
    }

    /// Colorize a raw status string.
    pub fn colorize_str(status: &str) -> String {
        match status {
            "SCHEDULED" => status.bright_yellow().bold().to_string(),
            "PASSED" => status.bright_green().bold().to_string(),
            "REJECTED" => status.bright_red().bold().to_string(),
            _ => status.to_string(),
        }
    }
}

impl Display for StageStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for StageStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SCHEDULED" => Ok(StageStatus::Scheduled),
            "PASSED" => Ok(StageStatus::Passed),
            "REJECTED" => Ok(StageStatus::Rejected),
            _ => Err(format!("Unknown stage status: {}", s)),
        }
    }
}

/// This struct defines a new interview stage that will be inserted into SQLite.
#[derive(Debug, Insertable)]
#[diesel(table_name = interview_stages)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewInterviewStage {
    /// The job application ID. References the record ID in SQLite.
    pub job_id: i32,
    /// The sequential stage number for this job.
    pub stage_number: i32,
    /// An optional name for the stage (e.g. "Phone Screen", "Technical Interview").
    pub name: Option<String>,
    /// The stage status (e.g. "SCHEDULED", "PASSED", "REJECTED").
    pub status: String,
    /// The date associated with this stage (formatted as YYYY/MM/DD).
    pub scheduled_date: String,
    /// Optional notes about this stage.
    pub notes: Option<String>,
    /// The timestamp at which this stage was created.
    pub created: String,
}

/// This struct defines the interview stage object returned from querying SQLite.
#[allow(dead_code)]
#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = interview_stages)]
#[diesel(check_for_backend(Sqlite))]
pub struct QueriedInterviewStage {
    /// The SQLite ID.
    pub id: i32,
    /// The job application ID. References the record ID in SQLite.
    pub job_id: i32,
    /// The sequential stage number for this job.
    pub stage_number: i32,
    /// An optional name for the stage (e.g. "Phone Screen", "Technical Interview").
    pub name: Option<String>,
    /// The stage status (e.g. "SCHEDULED", "PASSED", "REJECTED").
    pub status: String,
    /// The date associated with this stage (formatted as YYYY/MM/DD).
    pub scheduled_date: String,
    /// Optional notes about this stage.
    pub notes: Option<String>,
    /// The timestamp at which this stage was created.
    pub created: String,
}

impl Display for QueriedInterviewStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name_display = self
            .name
            .as_deref()
            .filter(|n| !n.is_empty())
            .map(|n| format!(": {}", n))
            .unwrap_or_default();
        write!(
            f,
            "Stage {}{} [{}] {}",
            self.stage_number, name_display, self.status, self.scheduled_date
        )
    }
}

/// This struct defines an updated interview stage that will overwrite an existing one in SQLite.
#[derive(Debug, Default, AsChangeset)]
#[diesel(table_name = interview_stages)]
#[diesel(check_for_backend(Sqlite))]
pub struct InterviewStageUpdate {
    /// An optional new name for the stage.
    pub name: Option<String>,
    /// An optional new status for the stage.
    pub status: Option<String>,
    /// An optional new date for the stage.
    pub scheduled_date: Option<String>,
    /// Optional new notes for the stage.
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_status_variants_returns_all_three() {
        let variants = StageStatus::variants();
        assert_eq!(variants.len(), 3);
    }

    #[test]
    fn test_stage_status_as_str() {
        assert_eq!(StageStatus::Scheduled.as_str(), "SCHEDULED");
        assert_eq!(StageStatus::Passed.as_str(), "PASSED");
        assert_eq!(StageStatus::Rejected.as_str(), "REJECTED");
    }

    #[test]
    fn test_stage_status_date_prompt() {
        assert_eq!(
            StageStatus::Scheduled.date_prompt(),
            "Select the scheduled date:"
        );
        assert_eq!(
            StageStatus::Passed.date_prompt(),
            "Select the passed date:"
        );
        assert_eq!(
            StageStatus::Rejected.date_prompt(),
            "Select the rejected date:"
        );
    }

    #[test]
    fn test_stage_status_colorize_str_known_statuses() {
        let scheduled = StageStatus::colorize_str("SCHEDULED");
        assert!(scheduled.contains("SCHEDULED"));

        let passed = StageStatus::colorize_str("PASSED");
        assert!(passed.contains("PASSED"));

        let rejected = StageStatus::colorize_str("REJECTED");
        assert!(rejected.contains("REJECTED"));
    }

    #[test]
    fn test_stage_status_colorize_str_unknown_returns_plain() {
        assert_eq!(StageStatus::colorize_str("UNKNOWN"), "UNKNOWN");
    }

    #[test]
    fn test_stage_status_display() {
        assert_eq!(format!("{}", StageStatus::Scheduled), "SCHEDULED");
        assert_eq!(format!("{}", StageStatus::Passed), "PASSED");
        assert_eq!(format!("{}", StageStatus::Rejected), "REJECTED");
    }

    #[test]
    fn test_stage_status_from_str() {
        assert!(matches!(
            "SCHEDULED".parse::<StageStatus>(),
            Ok(StageStatus::Scheduled)
        ));
        assert!(matches!(
            "PASSED".parse::<StageStatus>(),
            Ok(StageStatus::Passed)
        ));
        assert!(matches!(
            "REJECTED".parse::<StageStatus>(),
            Ok(StageStatus::Rejected)
        ));
    }

    #[test]
    fn test_stage_status_from_str_case_insensitive() {
        assert!(matches!(
            "scheduled".parse::<StageStatus>(),
            Ok(StageStatus::Scheduled)
        ));
        assert!(matches!(
            "Passed".parse::<StageStatus>(),
            Ok(StageStatus::Passed)
        ));
        assert!(matches!(
            "rEjEcTeD".parse::<StageStatus>(),
            Ok(StageStatus::Rejected)
        ));
    }

    #[test]
    fn test_stage_status_from_str_invalid() {
        let result = "INVALID".parse::<StageStatus>();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown stage status: INVALID");
    }

    #[test]
    fn test_queried_interview_stage_display_with_name() {
        let stage = QueriedInterviewStage {
            id: 1,
            job_id: 1,
            stage_number: 1,
            name: Some("Phone Screen".to_string()),
            status: "SCHEDULED".to_string(),
            scheduled_date: "2025/01/15".to_string(),
            notes: None,
            created: "2025-01-15".to_string(),
        };
        assert_eq!(
            format!("{}", stage),
            "Stage 1: Phone Screen [SCHEDULED] 2025/01/15"
        );
    }

    #[test]
    fn test_queried_interview_stage_display_without_name() {
        let stage = QueriedInterviewStage {
            id: 1,
            job_id: 1,
            stage_number: 2,
            name: None,
            status: "PASSED".to_string(),
            scheduled_date: "2025/02/01".to_string(),
            notes: Some("Went well".to_string()),
            created: "2025-02-01".to_string(),
        };
        assert_eq!(format!("{}", stage), "Stage 2 [PASSED] 2025/02/01");
    }

    #[test]
    fn test_queried_interview_stage_display_with_empty_name() {
        let stage = QueriedInterviewStage {
            id: 1,
            job_id: 1,
            stage_number: 3,
            name: Some("".to_string()),
            status: "REJECTED".to_string(),
            scheduled_date: "2025/03/01".to_string(),
            notes: None,
            created: "2025-03-01".to_string(),
        };
        assert_eq!(format!("{}", stage), "Stage 3 [REJECTED] 2025/03/01");
    }

    #[test]
    fn test_interview_stage_update_default() {
        let update = InterviewStageUpdate::default();
        assert!(update.name.is_none());
        assert!(update.status.is_none());
        assert!(update.scheduled_date.is_none());
        assert!(update.notes.is_none());
    }
}
