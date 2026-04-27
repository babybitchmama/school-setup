use chrono::{Local, NaiveDateTime};

/// Checks the due date against the current system time.
/// Returns (Option<Days Left>, Formatted Status String)
pub fn check_if_assignment_is_due(
    due_date_str: &str,
    submitted: bool,
    date_format: &str,
) -> (Option<i64>, String) {
    if submitted {
        return (None, "Submitted".to_string());
    }

    if let Ok(due_date) = NaiveDateTime::parse_from_str(due_date_str, date_format) {
        let now = Local::now().naive_local();
        let days_left = (due_date.date() - now.date()).num_days();

        if days_left < 0 {
            (Some(days_left), "Overdue!".to_string())
        } else {
            (Some(days_left), due_date_str.to_string())
        }
    } else {
        (None, "Invalid Date Format".to_string())
    }
}


/// Truncates a string and appends "..." if it exceeds the maximum length
pub fn generate_short_title(title: &str, max_len: usize) -> String {
    if title.len() <= max_len {
        title.to_string()
    } else {
        let cutoff = max_len.saturating_sub(3);
        format!("{}...", &title[..cutoff])
    }
}
