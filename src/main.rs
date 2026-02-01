//! `fetters` - a CLI tool for tracking your job applications.

mod cli;
mod commands;
mod config;
mod errors;
mod models;
mod repositories;
mod schema;
mod sqlite;
mod utils;

use clap::Parser;
use lazy_static::lazy_static;
use owo_colors::OwoColorize;

use crate::cli::{Cli, Command, ConfigOption, SprintOption, StageOption};
use crate::commands::add::add_job;
use crate::commands::config::edit_config;
use crate::commands::delete::delete_job;
use crate::commands::export::export_jobs;
use crate::commands::insights::show_insights;
use crate::commands::list::list_jobs;
use crate::commands::open::open_application;
use crate::commands::sprint::{
    create_new_sprint, set_sprint, show_all_sprints, show_current_sprint,
};
use crate::commands::stage::{add_stage, delete_stage, show_stage_tree, update_stage};
use crate::commands::update::update_job;
use crate::config::configuration::Config;
use crate::errors::FettersError;
use crate::repositories::{sprint::SprintRepository, statuses::StatusRepository};
use crate::sqlite::Database;
use crate::utils::migrations::run_migrations;

lazy_static! {
    /// ASCII art for `fetters`.
    static ref ASCII_ART: &'static [u8; 695] = include_bytes!("../art.txt");
}

/// Run `fetters`.
fn main() -> Result<(), FettersError> {
    let config = Config::load_or_create()?;
    let mut database = Database::new_connection(&config.db_path)?;

    run_migrations(&mut database.connection)?;

    // Ensure the default statuses are stored in the `statuses` SQLite table.
    let mut status_repo = StatusRepository {
        connection: &mut database.connection,
    };
    status_repo.seed_statuses()?;

    let mut sprint_repo = SprintRepository {
        connection: &mut database.connection,
    };
    let current_sprint = sprint_repo.get_current_sprint(&config.current_sprint)?;

    let cli = Cli::parse();

    match cli.command {
        Command::Add { company } => {
            if let Err(error) = add_job(&mut database.connection, &company, &current_sprint) {
                println!("{}", error.red().bold());
            }
        }
        Command::Banner => println!("{}", String::from_utf8_lossy(&ASCII_ART[..]).red().bold()),
        Command::Config(config_option) => match config_option {
            ConfigOption::Edit => {
                if let Err(error) = edit_config() {
                    println!("{}", error.red().bold());
                }
            }
            ConfigOption::Show => {
                println!("{config:#?}");
            }
        },
        Command::Delete(mut query_args) => {
            if let Err(error) =
                delete_job(&mut database.connection, &mut query_args, &current_sprint)
            {
                println!("{}", error.red().bold());
            }
        }
        Command::Export(mut export_args) => {
            if let Err(error) =
                export_jobs(&mut database.connection, &mut export_args, &current_sprint)
            {
                println!("{}", error.red().bold());
            }
        }
        Command::Insights => {
            if let Err(error) = show_insights(&mut database.connection, &current_sprint) {
                println!("{}", error.red().bold());
            }
        }
        Command::List(query_args) => {
            if let Err(error) = list_jobs(&mut database.connection, &query_args, &current_sprint) {
                println!("{}", error.red().bold());
            }
        }
        Command::Open(mut query_args) => {
            if let Err(error) =
                open_application(&mut database.connection, &mut query_args, &current_sprint)
            {
                println!("{}", error.red().bold());
            }
        }
        Command::Sprint(sprint_option) => match sprint_option {
            SprintOption::Current => {
                show_current_sprint(current_sprint);
            }
            SprintOption::New { name } => {
                if let Err(error) =
                    create_new_sprint(&mut database.connection, &name, config, &current_sprint)
                {
                    println!("{}", error.red().bold());
                }
            }
            SprintOption::ShowAll => {
                if let Err(error) = show_all_sprints(&mut database.connection) {
                    println!("{}", error.red().bold());
                }
            }
            SprintOption::Set => {
                if let Err(error) = set_sprint(&mut database.connection, config, &current_sprint) {
                    println!("{}", error.red().bold());
                }
            }
        },
        Command::Stage(stage_option) => match stage_option {
            StageOption::Add(mut query_args) => {
                if let Err(error) =
                    add_stage(&mut database.connection, &mut query_args, &current_sprint)
                {
                    println!("{}", error.red().bold());
                }
            }
            StageOption::Delete(mut query_args) => {
                if let Err(error) =
                    delete_stage(&mut database.connection, &mut query_args, &current_sprint)
                {
                    println!("{}", error.red().bold());
                }
            }
            StageOption::Tree(mut query_args) => {
                if let Err(error) =
                    show_stage_tree(&mut database.connection, &mut query_args, &current_sprint)
                {
                    println!("{}", error.red().bold());
                }
            }
            StageOption::Update(mut query_args) => {
                if let Err(error) =
                    update_stage(&mut database.connection, &mut query_args, &current_sprint)
                {
                    println!("{}", error.red().bold());
                }
            }
        },
        Command::Update(mut query_args) => {
            if let Err(error) =
                update_job(&mut database.connection, &mut query_args, &current_sprint)
            {
                println!("{}", error.red().bold());
            }
        }
    }

    Ok(())
}
