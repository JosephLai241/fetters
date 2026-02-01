//! Contains all models for job applications.

use std::fmt::{self, Display, Formatter};

use diesel::sqlite::Sqlite;
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use owo_colors::OwoColorize;
use tabled::Tabled;
use tabled::derive::display;

use crate::schema::jobs;

/// This struct defines the job object returned from querying SQLite.
#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = jobs)]
#[diesel(check_for_backend(Sqlite))]
pub struct QueriedJob {
    /// The SQLite ID.
    pub id: i32,
    /// The timestamp at which this job application was created.
    pub created: String,
    /// The name of the company.
    pub company_name: String,
    /// The job title.
    pub title_id: i32,
    /// The application status.
    pub status_id: i32,
    /// The link to the job application.
    pub link: Option<String>,
    /// Any notes about this job application.
    pub notes: Option<String>,
    /// The sprint ID. References the record ID in SQLite.
    pub sprint_id: i32,
}

/// This struct defines a new job application that will be inserted into SQLite.
#[derive(Debug, Insertable)]
#[diesel(table_name = jobs)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewJob<'a> {
    /// The name of the company.
    pub company_name: &'a str,
    /// The timestamp at which this job application was created.
    pub created: String,
    /// The job title ID. References the record ID in SQLite.
    pub title_id: i32,
    /// The application status ID. References the record ID in SQLite.
    pub status_id: i32,
    /// The link to the job application.
    pub link: Option<&'a str>,
    /// Any notes about this job application.
    pub notes: Option<&'a str>,
    /// The sprint ID. References the record ID in SQLite.
    pub sprint_id: i32,
}

/// This struct defines an updated job application that will overwrite an existing one in SQLite.
#[derive(Debug, Default, AsChangeset)]
#[diesel(table_name = jobs)]
#[diesel(check_for_backend(Sqlite))]
pub struct JobUpdate<'a> {
    /// The name of the company.
    pub company_name: Option<&'a str>,
    /// The job title ID. References the record ID in SQLite.
    pub title_id: Option<i32>,
    /// The application status ID. References the record ID in SQLite.
    pub status_id: Option<i32>,
    /// The link to the job application.
    pub link: Option<&'a str>,
    /// Any notes about this job application.
    pub notes: Option<&'a str>,
    /// The sprint ID. References the record ID in SQLite.
    pub sprint_id: Option<i32>,
}

/// This struct defines a job application with the title, status, and sprint name after querying
/// SQLite for those fields based on their record IDs and is used when displaying job applications
/// in tables.
#[derive(Clone, Debug, Queryable, Tabled)]
pub struct TabledJob {
    /// The SQLite ID.
    #[tabled(rename = "ID")]
    pub id: i32,
    /// The timestamp at which this job application was created.
    #[tabled(rename = "Created")]
    pub created: String,
    /// The name of the company.
    #[tabled(rename = "Company Name")]
    pub company_name: String,
    /// The job title.
    #[tabled(rename = "Title")]
    #[tabled(display("display::option", "N/A"))]
    pub title: Option<String>,
    /// The application status.
    #[tabled(rename = "Status")]
    #[tabled(display("display::option", "N/A"))]
    pub status: Option<String>,
    /// The number of interview stages tracked for this job application.
    #[tabled(rename = "Num Stages")]
    #[tabled(display("display::option", ""))]
    pub stages: Option<i32>,
    /// The link to the job application.
    #[tabled(rename = "Link")]
    #[tabled(display("display::option", "N/A"))]
    pub link: Option<String>,
    /// Any notes about this job application.
    #[tabled(rename = "Notes")]
    #[tabled(display("display::option", "N/A"))]
    pub notes: Option<String>,
}

impl TabledJob {
    /// Colorize a string based on the `status` field of the job application.
    fn colorize_field(&self, field_name: &str) -> String {
        if let Some(ref status) = self.status {
            match status {
                val if val == "GHOSTED" => {
                    return field_name.white().bold().to_string();
                }
                val if val == "HIRED" => return field_name.green().bold().to_string(),
                val if val == "IN PROGRESS" => return field_name.yellow().bold().to_string(),
                val if val == "NOT HIRING ANYMORE" => {
                    return field_name.fg_rgb::<201, 201, 201>().to_string();
                }
                val if val == "OFFER RECEIVED" => return field_name.magenta().bold().to_string(),
                val if val == "PENDING" => return field_name.blue().bold().to_string(),
                val if val == "REJECTED" => return field_name.red().bold().to_string(),
                _ => return field_name.to_string(),
            }
        }

        field_name.to_string()
    }

    /// Convert the struct to a row of strings to write to a spreadsheet when exporting job
    /// applications.
    pub fn convert_to_row(&self) -> Vec<String> {
        vec![
            self.created.clone(),
            self.company_name.clone(),
            self.title.clone().unwrap_or("N/A".to_string()),
            self.status.clone().unwrap_or("N/A".to_string()),
            self.link.clone().unwrap_or("".to_string()),
            self.notes.clone().unwrap_or("".to_string()),
        ]
    }
}

impl Display for TabledJob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ID: {} | Company: {} | Title: {} | Status: {}",
            self.id.white().bold(),
            self.colorize_field(&self.company_name),
            self.colorize_field(&self.title.clone().unwrap_or("".to_string())),
            self.colorize_field(&self.status.clone().unwrap_or("".to_string()))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tabled_job(status: Option<&str>) -> TabledJob {
        TabledJob {
            id: 1,
            created: "2025-01-15".to_string(),
            company_name: "Acme Corp".to_string(),
            title: Some("Software Engineer".to_string()),
            status: status.map(|s| s.to_string()),
            stages: Some(2),
            link: Some("https://example.com/apply".to_string()),
            notes: Some("Great opportunity".to_string()),
        }
    }

    #[test]
    fn test_convert_to_row_with_all_fields() {
        let job = make_tabled_job(Some("PENDING"));
        let row = job.convert_to_row();
        assert_eq!(row.len(), 6);
        assert_eq!(row[0], "2025-01-15");
        assert_eq!(row[1], "Acme Corp");
        assert_eq!(row[2], "Software Engineer");
        assert_eq!(row[3], "PENDING");
        assert_eq!(row[4], "https://example.com/apply");
        assert_eq!(row[5], "Great opportunity");
    }

    #[test]
    fn test_convert_to_row_with_none_fields() {
        let job = TabledJob {
            id: 1,
            created: "2025-01-15".to_string(),
            company_name: "Test Co".to_string(),
            title: None,
            status: None,
            stages: None,
            link: None,
            notes: None,
        };
        let row = job.convert_to_row();
        assert_eq!(row[2], "N/A");
        assert_eq!(row[3], "N/A");
        assert_eq!(row[4], "");
        assert_eq!(row[5], "");
    }

    #[test]
    fn test_colorize_field_with_no_status() {
        let job = TabledJob {
            id: 1,
            created: "2025-01-15".to_string(),
            company_name: "Test".to_string(),
            title: None,
            status: None,
            stages: None,
            link: None,
            notes: None,
        };
        assert_eq!(job.colorize_field("test"), "test");
    }

    #[test]
    fn test_colorize_field_with_unknown_status() {
        let job = make_tabled_job(Some("UNKNOWN_STATUS"));
        let result = job.colorize_field("test");
        assert_eq!(result, "test");
    }

    #[test]
    fn test_colorize_field_with_known_statuses() {
        let statuses = vec![
            "GHOSTED",
            "HIRED",
            "IN PROGRESS",
            "NOT HIRING ANYMORE",
            "OFFER RECEIVED",
            "PENDING",
            "REJECTED",
        ];
        for status in statuses {
            let job = make_tabled_job(Some(status));
            let result = job.colorize_field("test");
            assert!(
                result.contains("test"),
                "colorize_field should contain the input for status {}",
                status
            );
        }
    }

    #[test]
    fn test_tabled_job_display_contains_company() {
        let job = make_tabled_job(Some("PENDING"));
        let display = format!("{}", job);
        assert!(display.contains("Acme Corp"));
    }

    #[test]
    fn test_job_update_default() {
        let update = JobUpdate::default();
        assert!(update.company_name.is_none());
        assert!(update.title_id.is_none());
        assert!(update.status_id.is_none());
        assert!(update.link.is_none());
        assert!(update.notes.is_none());
        assert!(update.sprint_id.is_none());
    }
}
