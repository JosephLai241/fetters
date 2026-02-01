```
                            _
      .::::::::::.        -(_)====u         .::::::::::.
    .::::''''''::::.                      .::::''''''::::.
  .:::'          `::::....          ....::::'          `:::.
 .::'             `:::::::|        |:::::::'             `::.
.::|               |::::::|_ ___ __|::::::|               |::.
`--'               |::::::|_()__()_|::::::|               `--'
 :::               |::-o::|        |::o-::|               :::
 `::.             .|::::::|        |::::::|.             .::'
  `:::.          .::\-----'        `-----/::.          .:::'
    `::::......::::'                      `::::......::::'
      `::::::::::'                          `::::::::::'
                           fetters
```

[![GitHub Workflow Status](https://github.com/JosephLai241/fetters/actions/workflows/rust.yml/badge.svg)](https://github.com/JosephLai241/fetters/actions/workflows/rust.yml)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/JosephLai241/fetters)

# Table of Contents

- [Introduction](#introduction)
  - [What Does It Do?](#what-does-it-do)
- [Installation](#installation)
  - [`cargo install`](#cargo-install)
  - [Compile From Source](#compile-from-source)
- [Stored Attributes](#stored-attributes)
- [Job Sprints](#job-sprints)
- [Walkthrough](#walkthrough)
  - [Managing Job Sprints](#managing-job-sprints)
    - [Creating a New Sprint](#creating-a-new-sprint)
    - [Show Current Job Sprint](#show-current-job-sprint)
    - [Show All Job Sprints](#show-all-job-sprints)
    - [Switch to a Different Sprint](#switch-to-a-different-sprint)
  - [Adding a Job](#adding-a-job)
  - [Updating or Deleting a Job](#updating-or-deleting-a-job)
  - [Listing/Searching Jobs](#listingsearching-jobs)
  - [Managing Interview Stages](#managing-interview-stages)
    - [Adding a Stage](#adding-a-stage)
    - [Viewing the Stage Tree](#viewing-the-stage-tree)
    - [Updating a Stage](#updating-a-stage)
    - [Deleting a Stage](#deleting-a-stage)
  - [Display Job Insights](#display-job-insights)
  - [Opening Links](#opening-links)
  - [Exporting Jobs to XLSX](#exporting-jobs-to-xlsx)
- [Conclusion](#conclusion)

# Introduction

> [!IMPORTANT]
> **Requires SQLite v3.35.0+ installed on your system.**

`fetters` is a command-line tool that helps you track your job applications all in one place. The process of adding, updating, searching, or deleting job applications is very simple and fast.

You can create different "job sprints" (phases when you are looking for a new job) with this program and track the number of applications you sent during each period, providing a singular tool and space for you to track them.

## What Does It Do?

This program stores job applications, job titles, and sprints into a local SQLite database. This SQLite database is automatically created for you on the initial run and is stored in the project directory on your machine (see the documentation for [`directories::ProjectDirs::data_dir()`][projectdirs documentation] for the exact path on your platform).

This program enables you to:

- Add, update, and delete job applications from the database.
- List/search for tracked job applications. Each entry is color-coded based on the application status.
- Conveniently open links to job applications in your browser (given a URL) or document viewer (given a local filepath).
- Track interview stages for each job application with a color-coded tree view.
- Display application insights (ie. How many applications are in progress, how many have been rejected, how many are in pending status, etc.).
- Group job applications by sprints.
- Export tracked job applications to an XLSX file.

# Installation

## `cargo install`

You can run `cargo install` to install this on your machine:

```
cargo install fetters
```

## Compile From Source

You can compile this program from source with the following commands:

```
$ git clone https://www.github.com/JosephLai241/fetters
$ cd fetters
$ cargo build --release
```

To check if it built correctly:

```
$ ./target/release/fetters -V
```

You can then move the `fetters` binary to another directory so you do not have to type that path to run it. Check if the binary was moved correctly:

```
$ mv target/release/fetters /some/directory/
$ cd /some/directory
$ ./fetters -V
```

# Stored Attributes

Each record stores the following fields:

- Created timestamp (`YYYY-MM-DD HH:MM:SS`)
- Company name
- Job title
- Application status
- [Optional] Link to the application
- [Optional] Notes
- Job Sprint

The job status is color-coded in the table. Here is a table mapping each status to its color:

| Status             | Color     |
| ------------------ | --------- |
| GHOSTED            | Gray      |
| HIRED              | Green     |
| IN PROGRESS        | Yellow    |
| NOT HIRING ANYMORE | Dark Gray |
| OFFER RECEIVED     | Magenta   |
| PENDING            | Blue      |
| REJECTED           | Red       |

# Job Sprints

A job sprint is a period of time when you are actively submitting job applications. `fetters` allows you to organize your job applications into sprints so that it is easy to tell during which time period an application was submitted.

See the [Managing Job Sprints](#managing-job-sprints) section for more details.

# Walkthrough

## Managing Job Sprints

You can configure different job sprints to group job applications based on periods of time in which you are actively submitting job applications.

**A new job sprint will be created for you on the initial run. You do not have to worry about managing sprints if you don't plan on grouping your job applications by sprint.**

Job sprints are labeled with the date on which they are created (`YYYY-MM-DD`) by default but can be overridden with a custom name when creating a new sprint.

### Creating a New Sprint

The default name for a sprint is the current date (`YYYY-MM-DD`). You can optionally override the name of the sprint by providing the `-n/--name` flag.

Run the following command to create a new sprint:

```
fetters sprint new (-n <NAME>)
```

An error will be raised if you try to create a new sprint but there is already another sprint with an identical name.

<img width="1765" height="943" alt="image" src="https://github.com/user-attachments/assets/cc537948-1650-489e-bc44-234d10956718" />

### Show Current Job Sprint

Run the following command to show the current job sprint:

```
fetters sprint current
```

This will display a table containing the sprint name, start date, end date (if applicable), and the total number of applications in the sprint.

<img width="1765" height="943" alt="image" src="https://github.com/user-attachments/assets/0a30e6d0-585f-4fe8-b081-2a40af8c95e2" />

### Show All Job Sprints

Run the following command to show all job sprints:

```
fetters sprint show-all
```

Like the `current` subcommand, this will display a table containing all sprints, start dates, end dates (if applicable), and the total number of applications tracked in each sprint.

<img width="1765" height="943" alt="image" src="https://github.com/user-attachments/assets/3eded37d-4968-4937-8da5-c482a6754f51" />

### Switch to a Different Sprint

Run the following command to switch to or set a different sprint:

```
fetters sprint set
```

A select menu will appear and the selected sprint will be used to track applications until you decide to switch to a different sprint or create a new one.

<img width="1765" height="943" alt="image" src="https://github.com/user-attachments/assets/fcd06558-ff31-438e-a85e-b5d9064d1083" />

## Adding a Job

> [!NOTE]
>
> If you are utilizing [different sprints](#managing-job-sprints), the job application will be added to your current sprint.

Run the following command to track a new job application:

```
fetters add <COMPANY_NAME>
```

> [!TIP]
> Use quotes around the company name if it is more than one word or contains special terminal characters. For example, `&` is used to run a command asynchronously (running in the background) in a Bash terminal. Running `fetters add H&M` will cause problems for you if you do not wrap `H&M` in quotes.

A series of `inquire` prompts will show to set the job title, application status, link, and any notes.

<img width="1831" height="985" alt="image" src="https://github.com/user-attachments/assets/20513052-5b9e-4927-8c2d-89e7d4cd8d3d" />

## Updating or Deleting a Job

> [!NOTE]
>
> If you are utilizing [different sprints](#managing-job-sprints), these subcommands will search for jobs within your current sprint that match your query.

Run the following commands to update or delete a tracked job application:

```
fetters update
fetters delete
```

These commands support querying all stored attributes. Here is an example using all of the query options:

```
fetters update [OPTIONS]
fetters delete [OPTIONS]

Options:
  -c, --company <COMPANY_NAME>   Filter results by company name.
  -l, --link <LINK>              Filter results by links.
  -n, --notes <NOTES>            Filter results by notes.
      --sprint <SPRINT>          Filter results by sprint name.
  -s, --status <STATUS>          Filter results by application status.
      --stages [STAGES]          Filter by number of interview stages.
  -t, --title <TITLE>            Filter results by job title.
```

> [!TIP]
>
> All query options support partial text searching via the SQL `LIKE` operator.

The `delete` subcommand is very fast. A table of job applications (matching the query parameters or all applications if no query is provided) will be displayed, followed by an `inquire` prompt to select the job to delete.

<img width="1820" height="943" alt="image" src="https://github.com/user-attachments/assets/2f41af0a-4009-40f4-b419-af742f6a0787" />

The `update` subcommand will display a `MultiSelect` `inquire` prompt to select all the fields you want to update. `inquire` prompts will only show depending on the fields you have selected.

<img width="1820" height="943" alt="image" src="https://github.com/user-attachments/assets/42de1c6e-5e3c-4e16-ab50-03aaf7110b6f" />

## Listing/Searching Jobs

> [!NOTE]
>
> If you are utilizing [different sprints](#managing-job-sprints), this subcommand will search for jobs within your current sprint that match your query.

Run the following command to list or search job applications:

```
fetters list
```

Like the [`update` and `delete` subcommands](#updating-or-deleting-a-job), this also supports the same query options:

```
fetters list [OPTIONS]

Options:
  -c, --company <COMPANY_NAME>   Filter results by company name.
  -l, --link <LINK>              Filter results by links.
  -n, --notes <NOTES>            Filter results by notes.
      --sprint <SPRINT>          Filter results by sprint name.
  -s, --status <STATUS>          Filter results by application status.
      --stages [STAGES]          Filter by number of interview stages.
  -t, --title <TITLE>            Filter results by job title.
```

> [!TIP]
>
> All query options support partial text searching via the SQL `LIKE` operator.

Jobs matching your query parameters will be displayed in a table.

<img width="1820" height="943" alt="image" src="https://github.com/user-attachments/assets/41ba1eea-9502-4075-a0f7-52b40473e35d" />

## Managing Interview Stages

You can track interview stages for each job application. Each stage records a name (optional), status, date, and notes (optional). Stages are automatically numbered sequentially per job.

Stage statuses are color-coded in the tree view:

| Status    | Color  |
| --------- | ------ |
| SCHEDULED | Yellow |
| PASSED    | Green  |
| REJECTED  | Red    |

> [!NOTE]
>
> If you are utilizing [different sprints](#managing-job-sprints), these subcommands will search for jobs within your current sprint that match your query.

All stage subcommands support the same query options as `list`, `update`, and `delete` for selecting a job application:

```
fetters stage add [OPTIONS]
fetters stage delete [OPTIONS]
fetters stage tree [OPTIONS]
fetters stage update [OPTIONS]

Options:
  -c, --company <COMPANY_NAME>   Filter results by company name.
  -l, --link <LINK>              Filter results by links.
  -n, --notes <NOTES>            Filter results by notes.
      --sprint <SPRINT>          Filter results by sprint name.
  -s, --status <STATUS>          Filter results by application status.
      --stages [STAGES]          Filter by number of interview stages.
  -t, --title <TITLE>            Filter results by job title.
```

> [!TIP]
>
> All query options support partial text searching via the SQL `LIKE` operator.

### Adding a Stage

Run the following command to add an interview stage to a job application:

```
fetters stage add [OPTIONS]
```

You will be prompted to select a job, then enter a name, status, date, and notes for the stage. A tree preview of all stages (with the new stage highlighted) is displayed before confirmation.

<img width="1624" height="1061" alt="Screenshot 2026-01-31 at 19 44 59" src="https://github.com/user-attachments/assets/834356e8-bf95-426e-89be-8998f434657b" />

### Viewing the Stage Tree

Run the following command to display a tree of interview stages for a job application:

```
fetters stage tree [OPTIONS]
```

Only jobs with at least one tracked stage will be shown in the selection menu. The tree displays each stage with its status, date, and notes (if present).

<img width="1624" height="1061" alt="Screenshot 2026-01-31 at 19 52 44" src="https://github.com/user-attachments/assets/761ca7d6-da10-46f9-9f14-6a6573ffeae4" />

### Updating a Stage

Run the following command to update an interview stage:

```
fetters stage update [OPTIONS]
```

After selecting a job and stage, a `MultiSelect` prompt lets you choose which fields to update (name, status, date, notes). A tree preview with the updated stage highlighted is displayed before confirmation.

<img width="1624" height="1061" alt="Screenshot 2026-01-31 at 19 52 04" src="https://github.com/user-attachments/assets/92bdd288-2ac3-4334-b86b-58929647ebea" />

### Deleting a Stage

Run the following command to delete an interview stage:

```
fetters stage delete [OPTIONS]
```

After selecting a job and stage, a tree preview with the stage to be deleted highlighted in red is displayed before confirmation. Remaining stages are automatically renumbered after deletion.

<img width="1624" height="1061" alt="Screenshot 2026-01-31 at 19 52 33" src="https://github.com/user-attachments/assets/142909e8-f90f-4d80-b51b-e56757a9e159" />

## Display Job Insights

> [!NOTE]
>
> If you are utilizing [different sprints](#managing-job-sprints), this subcommand will display insights for your current sprint.

Run the following command to show job application insights:

```
fetters insights
```

<img width="1820" height="943" alt="image" src="https://github.com/user-attachments/assets/2c4404fa-9e52-49b5-a548-e052c4c29435" />

## Opening Links

> [!NOTE]
>
> If you are utilizing [different sprints](#managing-job-sprints), this subcommand will search for jobs within your current sprint that match your query.

Each record provides an optional link field. This field can be a URL to the job application (ie. `https://linkedin.com/jobs/view/...`) or a path to a local file (ie. a PDF or Word document).

Run the following command to open the URL or file:

```
fetters open
```

Like the `update`, `delete`, and `list` subcommands, this also supports the same query options:

```
fetters open [OPTIONS]

Options:
  -c, --company <COMPANY_NAME>   Filter results by company name.
  -l, --link <LINK>              Filter results by links.
  -n, --notes <NOTES>            Filter results by notes.
      --sprint <SPRINT>          Filter results by sprint name.
  -s, --status <STATUS>          Filter results by application status.
      --stages [STAGES]          Filter by number of interview stages.
  -t, --title <TITLE>            Filter results by job title.
```

Jobs matching your query parameters will be displayed in a table. Once a job is selected, the link will be opened in your default browser or document viewer based on the file type.

<img width="2463" height="1279" alt="image" src="https://github.com/user-attachments/assets/d77b362c-0755-442c-8dc1-cc8d0fe276a3" />

## Exporting Jobs to XLSX

You can export all tracked job applications from a job sprint to an XLSX file. The rows will be color-coded based on the job application status, similar to how applications are listed with the `list` subcommand.

```
fetters export [OPTIONS]

Options:
  -d, --directory <DIRECTORY>   Export the spreadsheet to the given directory path.
  -f, --filename <FILENAME>     Set a filename for the exported file.
  -s, --sprint <SPRINT>         Select a sprint to export from. Defaults to the current sprint.
```

# Conclusion

I wish you the best of luck with finding a job. We all know how rough it is out there. I hope this little CLI tool helps you track your applications during the struggle and that you won't have to use this for too long until you find your next opportunity 🤞🏻.

[projectdirs documentation]: https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html#method.data_dir
