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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::job::TabledJob;

    #[test]
    fn test_get_status_color_known_statuses() {
        assert_eq!(get_status_color("GHOSTED"), "FF999999");
        assert_eq!(get_status_color("HIRED"), "FF00A36C");
        assert_eq!(get_status_color("IN PROGRESS"), "FFFFFF00");
        assert_eq!(get_status_color("NOT HIRING ANYMORE"), "FFC9C9C9");
        assert_eq!(get_status_color("OFFER RECEIVED"), "FFFF00FF");
        assert_eq!(get_status_color("PENDING"), "FF0096FF");
        assert_eq!(get_status_color("REJECTED"), "FFEE4B2B");
    }

    #[test]
    fn test_get_status_color_unknown_defaults_to_gray() {
        assert_eq!(get_status_color("UNKNOWN"), "FF999999");
        assert_eq!(get_status_color(""), "FF999999");
    }

    #[test]
    fn test_create_spreadsheet_with_sprint_name() {
        let sprint = Some("2025-01-15".to_string());
        let (book, sheet_name) = create_spreadsheet(&sprint).unwrap();
        assert_eq!(sheet_name, "Sprint: 2025-01-15");
        assert!(book.get_sheet_by_name(&sheet_name).is_some());
    }

    #[test]
    fn test_create_spreadsheet_without_sprint_name() {
        let sprint: Option<String> = None;
        let (book, sheet_name) = create_spreadsheet(&sprint).unwrap();
        assert_eq!(sheet_name, "Sprint: unknown");
        assert!(book.get_sheet_by_name(&sheet_name).is_some());
    }

    #[test]
    fn test_write_jobs_populates_spreadsheet() {
        let sprint = Some("test".to_string());
        let (mut book, sheet_name) = create_spreadsheet(&sprint).unwrap();

        let jobs = vec![
            TabledJob {
                id: 1,
                created: "2025-01-15".to_string(),
                company_name: "Acme".to_string(),
                title: Some("SWE".to_string()),
                status: Some("PENDING".to_string()),
                stages: None,
                link: Some("https://example.com".to_string()),
                notes: Some("Notes here".to_string()),
            },
            TabledJob {
                id: 2,
                created: "2025-01-16".to_string(),
                company_name: "Globex".to_string(),
                title: None,
                status: None,
                stages: None,
                link: None,
                notes: None,
            },
        ];

        write_jobs(&mut book, &sheet_name, jobs);

        let worksheet = book.get_sheet_by_name(&sheet_name).unwrap();

        // Verify headers exist in row 1
        assert_eq!(
            worksheet.get_cell((1, 1)).unwrap().get_value(),
            "Timestamp"
        );
        assert_eq!(
            worksheet.get_cell((2, 1)).unwrap().get_value(),
            "Company Name"
        );
        assert_eq!(worksheet.get_cell((3, 1)).unwrap().get_value(), "Title");
        assert_eq!(worksheet.get_cell((4, 1)).unwrap().get_value(), "Status");
        assert_eq!(worksheet.get_cell((5, 1)).unwrap().get_value(), "Link");
        assert_eq!(worksheet.get_cell((6, 1)).unwrap().get_value(), "Notes");

        // Verify first data row
        assert_eq!(
            worksheet.get_cell((1, 2)).unwrap().get_value(),
            "2025-01-15"
        );
        assert_eq!(worksheet.get_cell((2, 2)).unwrap().get_value(), "Acme");
        assert_eq!(worksheet.get_cell((3, 2)).unwrap().get_value(), "SWE");
        assert_eq!(worksheet.get_cell((4, 2)).unwrap().get_value(), "PENDING");

        // Verify second data row
        assert_eq!(
            worksheet.get_cell((1, 3)).unwrap().get_value(),
            "2025-01-16"
        );
        assert_eq!(worksheet.get_cell((2, 3)).unwrap().get_value(), "Globex");
    }
}
