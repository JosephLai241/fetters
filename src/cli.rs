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
    /// List job applications. All applications are listed if no query arguments are provided.
    List(QueryArgs),
    /// Open the web link in your default browser or the local file associated with a job application.
    Open(QueryArgs),
    /// Configuration options for job sprints.
    #[command(subcommand)]
    Sprint(SprintOption),
    /// Manage interview stages for a particular job application.
    #[command(subcommand)]
    Stage(StageOption),
    /// Update a tracked job application.
    Update(QueryArgs),
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
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "0",
        help = "Filter by number of interview stages. Without a value, shows jobs with any stages. With a number, shows jobs with that exact count."
    )]
    pub stages: Option<i32>,
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

/// All subcommands for managing interview stages for a particular job application.
#[derive(Debug, Subcommand)]
pub enum StageOption {
    /// Add a new interview stage to an application.
    Add(QueryArgs),
    /// Delete an interview stage from an application.
    Delete(QueryArgs),
    /// Display a tree of interview stages. Trees for all applications that have tracked stages are
    /// displayed if no query arguments are provided.
    Tree(QueryArgs),
    /// Update an interview stage for an application.
    Update(QueryArgs),
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_add_command() {
        let cli = Cli::try_parse_from(["fetters", "add", "Google"]).unwrap();
        match cli.command {
            Command::Add { company } => assert_eq!(company, "Google"),
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_banner_command() {
        let cli = Cli::try_parse_from(["fetters", "banner"]).unwrap();
        assert!(matches!(cli.command, Command::Banner));
    }

    #[test]
    fn test_parse_list_command_with_no_args() {
        let cli = Cli::try_parse_from(["fetters", "list"]).unwrap();
        match cli.command {
            Command::List(args) => {
                assert!(args.company.is_none());
                assert!(args.status.is_none());
                assert!(args.title.is_none());
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_list_command_with_filters() {
        let cli = Cli::try_parse_from([
            "fetters", "list", "--company", "Google", "--status", "PENDING", "--title", "SWE",
        ])
        .unwrap();
        match cli.command {
            Command::List(args) => {
                assert_eq!(args.company.as_deref(), Some("Google"));
                assert_eq!(args.status.as_deref(), Some("PENDING"));
                assert_eq!(args.title.as_deref(), Some("SWE"));
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_delete_command() {
        let cli =
            Cli::try_parse_from(["fetters", "delete", "--company", "Meta"]).unwrap();
        match cli.command {
            Command::Delete(args) => assert_eq!(args.company.as_deref(), Some("Meta")),
            _ => panic!("Expected Delete command"),
        }
    }

    #[test]
    fn test_parse_update_command() {
        let cli =
            Cli::try_parse_from(["fetters", "update", "--company", "Apple"]).unwrap();
        match cli.command {
            Command::Update(args) => assert_eq!(args.company.as_deref(), Some("Apple")),
            _ => panic!("Expected Update command"),
        }
    }

    #[test]
    fn test_parse_insights_command() {
        let cli = Cli::try_parse_from(["fetters", "insights"]).unwrap();
        assert!(matches!(cli.command, Command::Insights));
    }

    #[test]
    fn test_parse_export_command() {
        let cli = Cli::try_parse_from([
            "fetters", "export", "-d", "/tmp", "-f", "export.xlsx",
        ])
        .unwrap();
        match cli.command {
            Command::Export(args) => {
                assert_eq!(args.directory.as_deref(), Some("/tmp"));
                assert_eq!(args.filename.as_deref(), Some("export.xlsx"));
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn test_parse_export_with_sprint() {
        let cli = Cli::try_parse_from([
            "fetters", "export", "-s", "my-sprint",
        ])
        .unwrap();
        match cli.command {
            Command::Export(args) => {
                assert_eq!(args.sprint.as_deref(), Some("my-sprint"));
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn test_parse_sprint_current() {
        let cli = Cli::try_parse_from(["fetters", "sprint", "current"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Sprint(SprintOption::Current)
        ));
    }

    #[test]
    fn test_parse_sprint_new_with_name() {
        let cli =
            Cli::try_parse_from(["fetters", "sprint", "new", "--name", "my-sprint"]).unwrap();
        match cli.command {
            Command::Sprint(SprintOption::New { name }) => {
                assert_eq!(name.as_deref(), Some("my-sprint"));
            }
            _ => panic!("Expected Sprint New"),
        }
    }

    #[test]
    fn test_parse_sprint_new_without_name() {
        let cli = Cli::try_parse_from(["fetters", "sprint", "new"]).unwrap();
        match cli.command {
            Command::Sprint(SprintOption::New { name }) => {
                assert!(name.is_none());
            }
            _ => panic!("Expected Sprint New"),
        }
    }

    #[test]
    fn test_parse_sprint_show_all() {
        let cli = Cli::try_parse_from(["fetters", "sprint", "show-all"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Sprint(SprintOption::ShowAll)
        ));
    }

    #[test]
    fn test_parse_sprint_set() {
        let cli = Cli::try_parse_from(["fetters", "sprint", "set"]).unwrap();
        assert!(matches!(cli.command, Command::Sprint(SprintOption::Set)));
    }

    #[test]
    fn test_parse_stage_add() {
        let cli =
            Cli::try_parse_from(["fetters", "stage", "add", "--company", "Google"]).unwrap();
        match cli.command {
            Command::Stage(StageOption::Add(args)) => {
                assert_eq!(args.company.as_deref(), Some("Google"));
            }
            _ => panic!("Expected Stage Add"),
        }
    }

    #[test]
    fn test_parse_stage_delete() {
        let cli = Cli::try_parse_from(["fetters", "stage", "delete"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Stage(StageOption::Delete(_))
        ));
    }

    #[test]
    fn test_parse_stage_tree() {
        let cli = Cli::try_parse_from(["fetters", "stage", "tree"]).unwrap();
        assert!(matches!(cli.command, Command::Stage(StageOption::Tree(_))));
    }

    #[test]
    fn test_parse_stage_update() {
        let cli = Cli::try_parse_from(["fetters", "stage", "update"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Stage(StageOption::Update(_))
        ));
    }

    #[test]
    fn test_parse_config_edit() {
        let cli = Cli::try_parse_from(["fetters", "config", "edit"]).unwrap();
        assert!(matches!(cli.command, Command::Config(ConfigOption::Edit)));
    }

    #[test]
    fn test_parse_config_show() {
        let cli = Cli::try_parse_from(["fetters", "config", "show"]).unwrap();
        assert!(matches!(cli.command, Command::Config(ConfigOption::Show)));
    }

    #[test]
    fn test_parse_open_command() {
        let cli =
            Cli::try_parse_from(["fetters", "open", "--company", "Netflix"]).unwrap();
        match cli.command {
            Command::Open(args) => assert_eq!(args.company.as_deref(), Some("Netflix")),
            _ => panic!("Expected Open command"),
        }
    }

    #[test]
    fn test_parse_list_with_stages_flag_no_value() {
        let cli = Cli::try_parse_from(["fetters", "list", "--stages"]).unwrap();
        match cli.command {
            Command::List(args) => assert_eq!(args.stages, Some(0)),
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_list_with_stages_flag_with_value() {
        let cli = Cli::try_parse_from(["fetters", "list", "--stages", "3"]).unwrap();
        match cli.command {
            Command::List(args) => assert_eq!(args.stages, Some(3)),
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_invalid_command_fails() {
        assert!(Cli::try_parse_from(["fetters", "nonexistent"]).is_err());
    }

    #[test]
    fn test_parse_missing_required_arg_fails() {
        assert!(Cli::try_parse_from(["fetters", "add"]).is_err());
    }

    #[test]
    fn test_query_args_default() {
        let args = QueryArgs::default();
        assert!(args.company.is_none());
        assert!(args.link.is_none());
        assert!(args.notes.is_none());
        assert!(args.sprint.is_none());
        assert!(args.status.is_none());
        assert!(args.title.is_none());
        assert!(args.stages.is_none());
    }
}
