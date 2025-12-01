//! Contains all CLI options.

use clap::{Parser, Subcommand};

/// Contains all CLI options for `fetters`.
#[derive(Debug, Parser)]
#[command(name = "fetters")]
#[command(about, version)]
pub struct Cli {
    /// Run a subcommand.
    #[command(subcommand)]
    pub command: Command,
}

/// Contains all subcommands for `fetters`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Track a new job application.
    Add {
        /// The name of the company.
        company: String,
    },
    /// Display the ASCII art.
    Banner,
    /// Configure `fetters` by opening its config file.
    #[command(subcommand)]
    Config(ConfigOption),
    /// Delete a tracked job application.
    Delete(QueryArgs),
    /// Export all tracked job applications from a job sprint to a spreadsheet.
    Export(ExportArgs),
    /// Show job application inslghts.
    Insights,
    /// List job applications.
    List(QueryArgs),
    /// Open the web link in your default browser or the local file associated with a job application.
    Open(QueryArgs),
    /// Configuration options for job sprints.
    #[command(subcommand)]
    Sprint(SprintOption),
    /// Update a tracked job application.
    Update(QueryArgs),
}

/// All flags you can use to query jobs.
#[derive(Debug, Default, Parser)]
pub struct QueryArgs {
    #[arg(
        short,
        long,
        help = "Filter results by company name. Supports searching with partial text."
    )]
    pub company: Option<String>,
    #[arg(
        short,
        long,
        help = "Filter results by links. Supports searching with partial text."
    )]
    pub link: Option<String>,
    #[arg(
        short,
        long,
        help = "Filter results by notes. Supports searching with partial text."
    )]
    pub notes: Option<String>,
    #[arg(
        long,
        help = "Filter results by sprint name. Supports searching with partial text."
    )]
    pub sprint: Option<String>,
    #[arg(
        short,
        long,
        help = "Filter results by application status. Supports searching with partial text."
    )]
    pub status: Option<String>,
    #[arg(
        short,
        long,
        help = "Filter results by job title. Supports searching with partial text."
    )]
    pub title: Option<String>,
}

/// All subcommands for interacting with the configuration file for `fetters`.
#[derive(Debug, Subcommand)]
pub enum ConfigOption {
    /// Edit the configuration file. You typically don't need to use this command as fetters will
    /// set these fields with other subcommands. However, this is available if you absolutely need
    /// to manually change values.
    Edit,
    /// Display the current configuration settings
    Show,
}

/// All subcommands for managing job sprints.
#[derive(Debug, Subcommand)]
pub enum SprintOption {
    /// Display the current sprint name.
    Current,
    /// Create a new job sprint.
    New {
        #[arg(short, long, help = "Override the default sprint name (YYYY-MM-DD).")]
        name: Option<String>,
    },
    /// Show all job sprints tracked by `fetters`.
    ShowAll,
    /// Set the current job sprint.
    Set,
}

/// All subcommands for exporting tracked jobs.
#[derive(Debug, Parser)]
pub struct ExportArgs {
    #[arg(
        short,
        long,
        help = "Export the spreadsheet to the given directory path. Defaults to the current directory if this is not provided."
    )]
    pub directory: Option<String>,

    #[arg(
        short,
        long,
        help = "Set a filename for the exported file. The '.xlsx' extension is automatically added if it is not provided. Defaults to '<DATE>-fetters-export-sprint-<SPRINT_NAME>.xlsx'"
    )]
    pub filename: Option<String>,

    #[arg(
        short,
        long,
        help = "Select a sprint to export from. Defaults to the current sprint."
    )]
    pub sprint: Option<String>,
}
