//! Contains a function called by the CLI when adding a new job.

use chrono::Local;
use diesel::sqlite::SqliteConnection;
use inquire::{Confirm, Select, Text};
use owo_colors::OwoColorize;

use crate::utils::prompt::get_inquire_config;
use crate::{errors::FettersError, utils::display::display_single_job};
use crate::{
    models::{
        job::{NewJob, TabledJob},
        sprint::QueriedSprint,
        status::QueriedStatus,
        title::NewTitle,
    },
    utils::titles::create_or_use_title,
};
use crate::{
    repositories::{job::JobRepository, statuses::StatusRepository, title::TitleRepository},
    utils::titles::TitleType,
};

/// Run the inquire menu to track a new job application.
pub fn add_job(
    connection: &mut SqliteConnection,
    company_name: &str,
    current_sprint: &QueriedSprint,
) -> Result<(), FettersError> {
    let title_type = create_or_use_title(connection)?;
    let status = select_status(connection)?;
    let link = input_link()?;
    let notes = input_notes()?;

    let created = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tabled_job = TabledJob {
        // NOTE: The ID is set to an arbitrary value to satisfy struct requirements.
        id: 0,
        created: created.clone(),
        company_name: company_name.to_string(),
        title: Some(match title_type {
            TitleType::NewTitle(ref title) => title.to_string(),
            TitleType::QueriedTitle(ref queried_title) => queried_title.name.to_string(),
        }),
        status: Some(status.name),
        stages: None,
        link: link.clone(),
        notes: notes.clone(),
    };

    display_single_job(&tabled_job);

    loop {
        match Confirm::new("Confirm new entry?")
            .with_default(true)
            .with_render_config(get_inquire_config())
            .prompt_skippable()?
        {
            Some(true) => {
                let title_id = match title_type {
                    TitleType::NewTitle(new_title) => {
                        let mut title_repo = TitleRepository { connection };
                        title_repo.add_title(NewTitle { name: &new_title })?.id
                    }
                    TitleType::QueriedTitle(queried_title) => queried_title.id,
                };
                let new_job = NewJob {
                    company_name,
                    created,
                    title_id,
                    status_id: status.id,
                    link: link.as_deref(),
                    notes: notes.as_deref(),
                    sprint_id: current_sprint.id,
                };

                let mut job_repo = JobRepository { connection };
                job_repo.add_job(new_job)?;

                println!(
                    "{}",
                    format!(
                        "\nCreated new entry for sprint [{}]!\n",
                        current_sprint.name
                    )
                    .green()
                    .bold()
                );

                return Ok(());
            }
            Some(false) => {
                println!("{}", "Cancelled.".red().bold());
                return Ok(());
            }
            None => println!("{}", "Invalid input, try again".red().bold()),
        }
    }
}
/// Select a job application status.
fn select_status(connection: &mut SqliteConnection) -> Result<QueriedStatus, FettersError> {
    let mut status_repo = StatusRepository { connection };
    let all_statuses = status_repo.get_all_statuses()?;

    let selected = Select::new("Select a status for this application:", all_statuses)
        .with_render_config(get_inquire_config())
        .prompt_skippable()?;

    if let Some(status) = selected {
        Ok(status)
    } else {
        Err(FettersError::UnknownError(
            "No selection was provided.".to_string(),
        ))
    }
}

/// Input an optional link to the job application.
fn input_link() -> Result<Option<String>, FettersError> {
    Ok(
        Text::new("[OPTIONAL] Enter a link to this job application:")
            .with_render_config(get_inquire_config())
            .prompt_skippable()?,
    )
}

/// Input optional notes for the job application.
fn input_notes() -> Result<Option<String>, FettersError> {
    Ok(
        Text::new("[OPTIONAL] Enter any notes for this job application:")
            .with_render_config(get_inquire_config())
            .prompt_skippable()?,
    )
}
