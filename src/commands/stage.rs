//! Contains functions called by the CLI when managing interview stages.

use chrono::{Local, NaiveDate};
use diesel::SqliteConnection;
use inquire::{Confirm, DateSelect, MultiSelect, Select, Text};
use owo_colors::OwoColorize;
use ptree::{TreeBuilder, print_tree};

use crate::{
    cli::QueryArgs,
    errors::FettersError,
    models::{
        job::TabledJob,
        sprint::QueriedSprint,
        stage::{
            InterviewStageUpdate, NewInterviewStage, QueriedInterviewStage, StageStatus,
        },
    },
    repositories::{job::JobRepository, stage::StageRepository},
    utils::{display::display_jobs, prompt::get_inquire_config},
};

/// Shared helper to select a job from query results.
fn select_job(
    connection: &mut SqliteConnection,
    query_args: &mut QueryArgs,
    current_sprint: &QueriedSprint,
) -> Result<Option<TabledJob>, FettersError> {
    let default_sprint = Some(current_sprint.name.clone());

    if query_args.sprint.is_none() {
        query_args.sprint = default_sprint;
    }

    let mut job_repo = JobRepository { connection };
    let matched_jobs = job_repo.list_jobs(query_args, current_sprint)?;

    if matched_jobs.is_empty() {
        return Err(FettersError::NoJobsAvailable(
            query_args
                .sprint
                .clone()
                .as_ref()
                .unwrap_or(&current_sprint.name)
                .to_string(),
        ));
    }

    display_jobs(
        &matched_jobs,
        query_args.sprint.as_ref().unwrap_or(&current_sprint.name),
    );

    Ok(Select::new("Select a job application:", matched_jobs)
        .with_render_config(get_inquire_config())
        .prompt_skippable()?)
}

/// Build and print a ptree for a job's interview stages.
fn build_stage_tree(
    job: &TabledJob,
    stages: &[QueriedInterviewStage],
    highlight_stage_id: Option<i32>,
    highlight_color: HighlightColor,
) {
    let root_label = format!(
        "{} - {}",
        job.company_name.white().bold(),
        job.title
            .as_deref()
            .unwrap_or("N/A")
            .bright_cyan()
            .bold()
    );

    let mut builder = TreeBuilder::new(root_label);

    for stage in stages {
        let is_highlighted = highlight_stage_id == Some(stage.id);

        let stage_label = match stage.name.as_deref() {
            Some(name) if !name.is_empty() => {
                format!("Stage {}: {}", stage.stage_number, name)
            }
            _ => format!("Stage {}", stage.stage_number),
        };

        let stage_label = if is_highlighted {
            match highlight_color {
                HighlightColor::Green => stage_label.green().bold().to_string(),
                HighlightColor::Red => stage_label.red().bold().to_string(),
            }
        } else {
            stage_label.white().bold().to_string()
        };

        builder.begin_child(stage_label);

        let status_display = if is_highlighted {
            match highlight_color {
                HighlightColor::Green => format!(
                    "[{}] {}",
                    stage.status.green().bold(),
                    stage.scheduled_date.green()
                ),
                HighlightColor::Red => format!(
                    "[{}] {}",
                    stage.status.red().bold(),
                    stage.scheduled_date.red()
                ),
            }
        } else {
            format!(
                "[{}] {}",
                StageStatus::colorize_str(&stage.status),
                stage.scheduled_date
            )
        };
        builder.add_empty_child(status_display);

        if let Some(ref notes) = stage.notes.as_deref().filter(|n| !n.is_empty()) {
            let notes_display = if is_highlighted {
                match highlight_color {
                    HighlightColor::Green => notes.green().to_string(),
                    HighlightColor::Red => notes.red().to_string(),
                }
            } else {
                notes.to_string()
            };
            builder.add_empty_child(notes_display);
        }

        builder.end_child();
    }

    let tree = builder.build();
    println!();
    print_tree(&tree).ok();
    println!();
}

/// Color used for highlighting a stage in the tree preview.
enum HighlightColor {
    /// Green highlight for new or updated stages.
    Green,
    /// Red highlight for stages being deleted.
    Red,
}

/// Add a new interview stage to a job application.
pub fn add_stage(
    connection: &mut SqliteConnection,
    query_args: &mut QueryArgs,
    current_sprint: &QueriedSprint,
) -> Result<(), FettersError> {
    let job = match select_job(connection, query_args, current_sprint)? {
        Some(job) => job,
        None => return Ok(()),
    };

    let mut stage_repo = StageRepository { connection };
    let next_stage_number = stage_repo.get_next_stage_number(job.id)?;

    let name = Text::new("[OPTIONAL] Enter a name for this stage (e.g. Phone Screen):")
        .with_render_config(get_inquire_config())
        .prompt_skippable()?;

    let status = match Select::new(
        "Select the status for this stage:",
        StageStatus::variants(),
    )
    .with_render_config(get_inquire_config())
    .prompt_skippable()?
    {
        Some(s) => s,
        None => return Ok(()),
    };

    let scheduled_date = match DateSelect::new(status.date_prompt())
        .with_render_config(get_inquire_config())
        .prompt_skippable()?
    {
        Some(d) => d.format("%Y/%m/%d").to_string(),
        None => return Ok(()),
    };

    let notes = Text::new("[OPTIONAL] Enter any notes for this stage:")
        .with_render_config(get_inquire_config())
        .prompt_skippable()?;

    let created = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Build a preview including existing stages plus the new one.
    let existing_stages = stage_repo.get_stages_for_job(job.id)?;
    let preview_stage = QueriedInterviewStage {
        id: -1, // Sentinel value for preview.
        job_id: job.id,
        stage_number: next_stage_number,
        name: name.clone(),
        status: status.to_string(),
        scheduled_date: scheduled_date.clone(),
        notes: notes.clone(),
        created: created.clone(),
    };

    let mut all_stages = existing_stages;
    all_stages.push(preview_stage);

    build_stage_tree(&job, &all_stages, Some(-1), HighlightColor::Green);

    match Confirm::new("Confirm new stage?")
        .with_default(true)
        .with_render_config(get_inquire_config())
        .prompt_skippable()?
    {
        Some(true) => {
            let new_stage = NewInterviewStage {
                job_id: job.id,
                stage_number: next_stage_number,
                name,
                status: status.to_string(),
                scheduled_date,
                notes,
                created,
            };

            let mut stage_repo = StageRepository { connection };
            stage_repo.add_stage(new_stage)?;

            println!(
                "{}",
                format!("\nAdded stage {} for {}!\n", next_stage_number, job.company_name)
                    .green()
                    .bold()
            );
        }
        Some(false) => {
            println!("{}", "Cancelled.".red().bold());
        }
        None => println!("{}", "Invalid input, try again".red().bold()),
    }

    Ok(())
}

/// Display a tree of interview stages for a job application.
pub fn show_stage_tree(
    connection: &mut SqliteConnection,
    query_args: &mut QueryArgs,
    current_sprint: &QueriedSprint,
) -> Result<(), FettersError> {
    // Only show jobs that have at least one stage.
    if query_args.stages.is_none() {
        query_args.stages = Some(0);
    }

    let job = match select_job(connection, query_args, current_sprint)? {
        Some(job) => job,
        None => return Ok(()),
    };

    let mut stage_repo = StageRepository { connection };
    let stages = stage_repo.get_stages_for_job(job.id)?;

    if stages.is_empty() {
        println!(
            "{}",
            format!("\nNo interview stages tracked for {}.\n", job.company_name)
                .yellow()
                .bold()
        );
        return Ok(());
    }

    build_stage_tree(&job, &stages, None, HighlightColor::Green);

    Ok(())
}

/// The fields of an interview stage that can be updated.
#[derive(Debug)]
enum UpdatableStageField {
    /// Update the stage name.
    Name,
    /// Update the stage status.
    Status,
    /// Update the stage date.
    ScheduledDate,
    /// Update the stage notes.
    Notes,
}

impl std::fmt::Display for UpdatableStageField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdatableStageField::Name => write!(f, "Name"),
            UpdatableStageField::Status => write!(f, "Status"),
            UpdatableStageField::ScheduledDate => write!(f, "Date"),
            UpdatableStageField::Notes => write!(f, "Notes"),
        }
    }
}

/// Update an interview stage for a job application.
pub fn update_stage(
    connection: &mut SqliteConnection,
    query_args: &mut QueryArgs,
    current_sprint: &QueriedSprint,
) -> Result<(), FettersError> {
    let job = match select_job(connection, query_args, current_sprint)? {
        Some(job) => job,
        None => return Ok(()),
    };

    let mut stage_repo = StageRepository { connection };
    let stages = stage_repo.get_stages_for_job(job.id)?;

    if stages.is_empty() {
        println!(
            "{}",
            format!("\nNo interview stages tracked for {}.\n", job.company_name)
                .yellow()
                .bold()
        );
        return Ok(());
    }

    let selected_stage = match Select::new("Select the stage to update:", stages.clone())
        .with_render_config(get_inquire_config())
        .prompt_skippable()?
    {
        Some(s) => s,
        None => return Ok(()),
    };

    let field_options = vec![
        UpdatableStageField::Name,
        UpdatableStageField::Status,
        UpdatableStageField::ScheduledDate,
        UpdatableStageField::Notes,
    ];

    let selections = match MultiSelect::new("Select the fields to update:", field_options)
        .with_render_config(get_inquire_config())
        .prompt_skippable()?
    {
        Some(s) => s,
        None => return Ok(()),
    };

    let mut stage_update = InterviewStageUpdate::default();

    for selection in &selections {
        match selection {
            UpdatableStageField::Name => {
                let current_name = selected_stage.name.as_deref().unwrap_or("");
                stage_update.name = Text::new("Enter a new name for this stage:")
                    .with_initial_value(current_name)
                    .with_render_config(get_inquire_config())
                    .prompt_skippable()?;
            }
            UpdatableStageField::Status => {
                if let Some(new_status) = Select::new(
                    "Select a new status:",
                    StageStatus::variants(),
                )
                .with_render_config(get_inquire_config())
                .prompt_skippable()?
                {
                    stage_update.status = Some(new_status.to_string());
                }
            }
            UpdatableStageField::ScheduledDate => {
                let starting_date = NaiveDate::parse_from_str(
                    &selected_stage.scheduled_date,
                    "%Y/%m/%d",
                )
                .unwrap_or_else(|_| Local::now().date_naive());

                let effective_status = stage_update
                    .status
                    .as_deref()
                    .unwrap_or(&selected_stage.status);
                let date_prompt = effective_status
                    .parse::<StageStatus>()
                    .map(|s| s.date_prompt())
                    .unwrap_or("Select a new date:");

                if let Some(new_date) = DateSelect::new(date_prompt)
                    .with_starting_date(starting_date)
                    .with_render_config(get_inquire_config())
                    .prompt_skippable()?
                {
                    stage_update.scheduled_date = Some(new_date.format("%Y/%m/%d").to_string());
                }
            }
            UpdatableStageField::Notes => {
                let current_notes = selected_stage.notes.as_deref().unwrap_or("");
                stage_update.notes = Text::new("Enter new notes for this stage:")
                    .with_initial_value(current_notes)
                    .with_render_config(get_inquire_config())
                    .prompt_skippable()?;
            }
        }
    }

    // Build preview with updated values.
    let preview_stages: Vec<QueriedInterviewStage> = stages
        .iter()
        .map(|s| {
            if s.id == selected_stage.id {
                QueriedInterviewStage {
                    id: s.id,
                    job_id: s.job_id,
                    stage_number: s.stage_number,
                    name: stage_update.name.clone().or(s.name.clone()),
                    status: stage_update
                        .status
                        .clone()
                        .unwrap_or(s.status.clone()),
                    scheduled_date: stage_update
                        .scheduled_date
                        .clone()
                        .unwrap_or(s.scheduled_date.clone()),
                    notes: stage_update.notes.clone().or(s.notes.clone()),
                    created: s.created.clone(),
                }
            } else {
                s.clone()
            }
        })
        .collect();

    build_stage_tree(&job, &preview_stages, Some(selected_stage.id), HighlightColor::Green);

    match Confirm::new("Confirm updates?")
        .with_default(true)
        .with_render_config(get_inquire_config())
        .prompt_skippable()?
    {
        Some(true) => {
            let mut stage_repo = StageRepository { connection };
            stage_repo.update_stage(selected_stage.id, stage_update)?;

            println!(
                "{}",
                format!(
                    "\nUpdated stage {} for {}!\n",
                    selected_stage.stage_number, job.company_name
                )
                .green()
                .bold()
            );
        }
        Some(false) => {
            println!("{}", "Cancelled.".red().bold());
        }
        None => println!("{}", "Invalid input, try again".red().bold()),
    }

    Ok(())
}

/// Delete an interview stage from a job application.
pub fn delete_stage(
    connection: &mut SqliteConnection,
    query_args: &mut QueryArgs,
    current_sprint: &QueriedSprint,
) -> Result<(), FettersError> {
    let job = match select_job(connection, query_args, current_sprint)? {
        Some(job) => job,
        None => return Ok(()),
    };

    let mut stage_repo = StageRepository { connection };
    let stages = stage_repo.get_stages_for_job(job.id)?;

    if stages.is_empty() {
        println!(
            "{}",
            format!("\nNo interview stages tracked for {}.\n", job.company_name)
                .yellow()
                .bold()
        );
        return Ok(());
    }

    let selected_stage = match Select::new("Select the stage to delete:", stages.clone())
        .with_render_config(get_inquire_config())
        .prompt_skippable()?
    {
        Some(s) => s,
        None => return Ok(()),
    };

    build_stage_tree(&job, &stages, Some(selected_stage.id), HighlightColor::Red);

    match Confirm::new("Confirm deletion?")
        .with_default(true)
        .with_render_config(get_inquire_config())
        .prompt_skippable()?
    {
        Some(true) => {
            let mut stage_repo = StageRepository { connection };
            let deleted = stage_repo.delete_stage(selected_stage.id)?;
            stage_repo.renumber_stages(deleted.job_id)?;

            println!(
                "{}",
                format!(
                    "\nDeleted stage {} from {}!\n",
                    selected_stage.stage_number, job.company_name
                )
                .green()
                .bold()
            );
        }
        Some(false) => {
            println!("{}", "Cancelled.".red().bold());
        }
        None => println!("{}", "Invalid input, try again".red().bold()),
    }

    Ok(())
}
