//! Contains utilities for displaying job applications.

use owo_colors::OwoColorize;
use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Color, Modify, Panel, Remove, Style, Width,
        location::Locator,
        object::{Columns, Rows},
    },
};

use crate::models::{insight::CountAndPercentage, job::TabledJob, sprint::QueriedSprint};

/// Display jobs in a table.
pub fn display_jobs(jobs: &Vec<TabledJob>, sprint_name: &str) {
    let mut table = Table::new(jobs);

    table
        .with(Style::blank())
        .with(Panel::header(
            format!("{sprint_name} SPRINT [{} JOBS LISTED]", jobs.len())
                .green()
                .bold()
                .to_string(),
        ))
        .with(Modify::list(Rows::first(), Alignment::center()))
        .with(Modify::list(Rows::one(1), Color::FG_BRIGHT_CYAN))
        .modify(Rows::first(), Color::FG_BRIGHT_WHITE)
        .modify(Columns::one(2), Width::truncate(22).suffix("..."))
        .modify(Columns::one(3), Width::truncate(35).suffix("..."))
        .modify(Columns::one(6), Width::truncate(23).suffix("..."))
        .modify(Columns::one(7), Width::wrap(40).keep_words(true))
        .modify(Locator::content("GHOSTED"), Color::rgb_fg(133, 133, 133))
        .modify(Locator::content("HIRED"), Color::FG_BRIGHT_GREEN)
        .modify(Locator::content("IN PROGRESS"), Color::FG_BRIGHT_YELLOW)
        .modify(
            Locator::content("NOT HIRING ANYMORE"),
            Color::rgb_fg(117, 117, 117),
        )
        .modify(Locator::content("OFFER RECEIVED"), Color::FG_BRIGHT_MAGENTA)
        .modify(Locator::content("PENDING"), Color::FG_BRIGHT_BLUE)
        .modify(Locator::content("REJECTED"), Color::FG_BRIGHT_RED);

    println!("\n{table}\n");
}

/// Display a single job. This generic function works with any struct that implements `Tabled`.
pub fn display_single_job<T: Tabled>(job: T) {
    let mut table = Table::new([job]);
    table
        .with(Style::blank())
        .with(Remove::column(Columns::first()))
        .with(Modify::list(Rows::first(), Color::FG_BRIGHT_CYAN))
        .modify(Columns::one(1), Width::truncate(22).suffix("..."))
        .modify(Columns::one(2), Width::truncate(35).suffix("..."))
        .modify(Columns::one(5), Width::truncate(23).suffix("..."))
        .modify(Columns::one(6), Width::wrap(40).keep_words(true))
        .modify(Locator::content("GHOSTED"), Color::rgb_fg(133, 133, 133))
        .modify(Locator::content("HIRED"), Color::FG_BRIGHT_GREEN)
        .modify(Locator::content("IN PROGRESS"), Color::FG_BRIGHT_YELLOW)
        .modify(
            Locator::content("NOT HIRING ANYMORE"),
            Color::rgb_fg(117, 117, 117),
        )
        .modify(Locator::content("OFFER RECEIVED"), Color::FG_BRIGHT_MAGENTA)
        .modify(Locator::content("PENDING"), Color::FG_BRIGHT_BLUE)
        .modify(Locator::content("REJECTED"), Color::FG_BRIGHT_RED);

    println!("\n{table}\n");
}

/// Display sprint metadata.
pub fn display_sprint(queried_sprints: &Vec<QueriedSprint>, table_header: &str) {
    let mut table = Table::new(queried_sprints);

    table
        .with(Panel::header(table_header.green().bold().to_string()))
        .with(Modify::list(Rows::first(), Alignment::center()))
        .with(Modify::list(Rows::one(1), Color::FG_BRIGHT_BLUE))
        .with(Style::blank());

    println!("\n{table}\n");
}

/// Display insights information.
pub fn display_insights(
    count_and_percentages: Vec<CountAndPercentage>,
    table_header: &str,
    remove_current_sprint: bool,
) {
    let mut table = Table::new(count_and_percentages);

    if remove_current_sprint {
        table.with(Remove::column(Columns::one(2)));
    }

    table
        .with(Panel::header(table_header.green().bold().to_string()))
        .with(Modify::list(Rows::first(), Alignment::center()))
        .with(Modify::list(Rows::one(1), Color::FG_BRIGHT_BLUE))
        .with(Style::blank());

    println!("\n{table}\n");
}
