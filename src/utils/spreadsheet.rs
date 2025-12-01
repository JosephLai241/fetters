//! Contains utility functions for exporting job applications to a spreadsheet.

use umya_spreadsheet::{self, Spreadsheet};

use crate::{errors::FettersError, models::job::TabledJob};

/// Create a new spreadsheet for the provided sprint.
pub fn create_spreadsheet(sprint: &Option<String>) -> Result<(Spreadsheet, String), FettersError> {
    let sheet_name = format!(
        "Sprint: {}",
        sprint.clone().unwrap_or("unknown".to_string())
    );

    let mut book = umya_spreadsheet::new_file();
    book.set_sheet_name(0, &sheet_name)?;

    Ok((book, sheet_name))
}

/// Write exported jobs to the spreadsheet.
pub fn write_jobs(spreadsheet: &mut Spreadsheet, sheet_name: &str, jobs: Vec<TabledJob>) {
    let worksheet = spreadsheet.get_sheet_by_name_mut(sheet_name).unwrap();

    let headers = vec![
        "Timestamp",
        "Company Name",
        "Title",
        "Status",
        "Link",
        "Notes",
    ];
    for (col, header) in headers.into_iter().enumerate() {
        let coordinates = ((col + 1) as u32, 1);

        worksheet.get_cell_mut(coordinates).set_value(header);

        let style = worksheet.get_style_mut(coordinates);
        style.set_background_color("FF999999");
    }

    for (row_index, job) in jobs.iter().enumerate() {
        let row_number = (row_index + 2) as u32;
        let row_values = job.convert_to_row();

        let status_color = if let Some(status) = &job.status {
            get_status_color(status)
        } else {
            "FF999999".to_string()
        };

        for (column_index, data) in row_values.into_iter().enumerate() {
            let coordinates = ((column_index + 1) as u32, row_number);
            worksheet.get_cell_mut(coordinates).set_value(data);

            let style = worksheet.get_style_mut(coordinates);
            style.set_background_color(&status_color);
        }
    }
}

/// Returns a color based on the job application status.
fn get_status_color(status: &str) -> String {
    match status {
        "GHOSTED" => "FF999999".to_string(),
        "HIRED" => "FF00A36C".to_string(),
        "IN PROGRESS" => "FFFFFF00".to_string(),
        "NOT HIRING ANYMORE" => "FFC9C9C9".to_string(),
        "OFFER RECEIVED" => "FFFF00FF".to_string(),
        "PENDING" => "FF0096FF".to_string(),
        "REJECTED" => "FFEE4B2B".to_string(),
        _ => "FF999999".to_string(),
    }
}
