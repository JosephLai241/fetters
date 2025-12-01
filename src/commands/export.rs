//! Contains a function called by the CLI when exporting jobs from SQLite.

use std::env;

use chrono::Local;
use diesel::SqliteConnection;
use owo_colors::OwoColorize;

use crate::{
    cli::{ExportArgs, QueryArgs},
    errors::FettersError,
    models::sprint::QueriedSprint,
    repositories::job::JobRepository,
    utils::spreadsheet::{create_spreadsheet, write_jobs},
};

/// Export all jobs tracked for a given sprint.
pub fn export_jobs(
    connection: &mut SqliteConnection,
    export_args: &mut ExportArgs,
    current_sprint: &QueriedSprint,
) -> Result<(), FettersError> {
    let target_sprint = if export_args.sprint.is_none() {
        Some(current_sprint.name.clone())
    } else {
        export_args.sprint.clone()
    };

    let mut job_repo = JobRepository { connection };

    let query_args = QueryArgs {
        sprint: target_sprint.clone(),
        ..Default::default()
    };

    let matched_jobs = job_repo.list_jobs(&query_args, current_sprint)?;

    if matched_jobs.is_empty() {
        return Err(FettersError::NoJobsAvailable(
            query_args
                .sprint
                .clone()
                .as_ref()
                .unwrap_or(&current_sprint.name.clone())
                .to_string(),
        ));
    }

    let (mut spreadsheet, sheet_name) = create_spreadsheet(&target_sprint)?;
    write_jobs(&mut spreadsheet, &sheet_name, matched_jobs);

    let filename = if let Some(filename) = export_args.filename.clone() {
        if !filename.ends_with(".xlsx") {
            format!("{filename}.xlsx")
        } else {
            filename
        }
    } else {
        format!(
            "{}-fetters-export-sprint-{}.xlsx",
            Local::now().format("%Y-%m-%d"),
            target_sprint.clone().unwrap_or("unknown".to_string())
        )
    };

    let export_path = format!(
        "{}/{}",
        export_args
            .directory
            .clone()
            .unwrap_or(env::current_dir()?.to_string_lossy().to_string()),
        filename,
    );

    umya_spreadsheet::writer::xlsx::write(&spreadsheet, &export_path)?;

    println!(
        "{}",
        format!(
            "Successfully exported all jobs for sprint {} to path: {export_path}!",
            target_sprint.unwrap_or("unknown".to_string())
        )
        .green()
        .bold()
    );

    Ok(())
}
