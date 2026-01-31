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
