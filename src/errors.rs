//! Contains an enum encapsulating all errors that may occur while using `fetters`.

use thiserror::Error;

/// Contains variants for errors that may be raised throughout this program.
#[derive(Debug, Error)]
pub enum FettersError {
    /// Something went wrong when trying to get the application-specific directories.
    #[error("Could not retrieve system application directories!")]
    ApplicationError,

    /// Something went wrong when attempting to get the result after creating or updating a job in
    /// SQLite.
    #[error("Diesel query result error: {0}")]
    DieselResultError(#[from] diesel::result::Error),

    /// An IO error occurred.
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    /// Something went wrong when using the `Inquire` crate for prompts.
    #[error("Inquire error: {0}")]
    InquireError(#[from] inquire::error::InquireError),

    /// Something fucked up when running the SQLite migrations with `diesel_migrations`.
    #[error("Failed to run migrations!")]
    MigrationFailure,

    /// This error may be raised if the user tries to update or delete a job, but no job
    /// applications have been tracked for the current sprint.
    #[error("No job applications tracked for the current sprint [{0}]")]
    NoJobsAvailable(String),

    /// This error is used when a result returns an error message. This is currently used to
    /// propagate the error returned when attempting to call `book.set_sheet_name()`.
    #[error("Set sheet name error: {0}")]
    SheetNameError(String),

    /// This error may be raised if the user attempts to create two new sprints in the same day,
    /// causing a sprint naming conflict (all sprint names should be unique).
    #[error("There is already a sprint with name {0}. Try renaming the sprint.")]
    SprintNameConflict(String),

    /// Something went wrong when trying to connect to the SQLite database.
    #[error("Failed to connect to SQLite database: {0}")]
    SQLiteConnectionError(#[from] diesel::ConnectionError),

    /// Something went wrong when deserializing TOML.
    #[error("TOML deserialization error: {0}")]
    TOMLDeserializationError(#[from] toml::de::Error),

    /// Something went wrong when serializing TOML.
    #[error("TOML serialization error: {0}")]
    TOMLSerializationError(#[from] toml::ser::Error),

    /// An unknown error occurred.
    #[error("{0}")]
    UnknownError(String),

    /// Something fucked up when exporting job applications to XLSX.
    #[error("XLSX write error: {0}")]
    XLSXError(#[from] umya_spreadsheet::XlsxError),
}

impl From<&str> for FettersError {
    fn from(value: &str) -> Self {
        Self::SheetNameError(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_creates_sheet_name_error() {
        let error: FettersError = "bad sheet name".into();
        match error {
            FettersError::SheetNameError(msg) => assert_eq!(msg, "bad sheet name"),
            _ => panic!("Expected SheetNameError"),
        }
    }

    #[test]
    fn test_error_display_application_error() {
        let error = FettersError::ApplicationError;
        assert_eq!(
            format!("{}", error),
            "Could not retrieve system application directories!"
        );
    }

    #[test]
    fn test_error_display_migration_failure() {
        let error = FettersError::MigrationFailure;
        assert_eq!(format!("{}", error), "Failed to run migrations!");
    }

    #[test]
    fn test_error_display_no_jobs_available() {
        let error = FettersError::NoJobsAvailable("sprint-1".to_string());
        assert_eq!(
            format!("{}", error),
            "No job applications tracked for the current sprint [sprint-1]"
        );
    }

    #[test]
    fn test_error_display_sheet_name_error() {
        let error = FettersError::SheetNameError("bad name".to_string());
        assert_eq!(format!("{}", error), "Set sheet name error: bad name");
    }

    #[test]
    fn test_error_display_sprint_name_conflict() {
        let error = FettersError::SprintNameConflict("2025-01-15".to_string());
        assert_eq!(
            format!("{}", error),
            "There is already a sprint with name 2025-01-15. Try renaming the sprint."
        );
    }

    #[test]
    fn test_error_display_unknown_error() {
        let error = FettersError::UnknownError("something broke".to_string());
        assert_eq!(format!("{}", error), "something broke");
    }
}
