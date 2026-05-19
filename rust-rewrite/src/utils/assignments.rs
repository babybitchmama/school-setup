use chrono::{Local, NaiveDate};

pub fn check_if_assignment_is_due(
    due_date_str: &str,
    submitted: bool,
) -> (Option<i64>, String) {
    const ASSIGNMENT_DATE_FORMAT: &str = "%m-%d-%y";

    if submitted {
        return (None, "Submitted".to_string());
    }

    if let Ok(due_date) = NaiveDate::parse_from_str(due_date_str, ASSIGNMENT_DATE_FORMAT) {
        let now: NaiveDate = Local::now().date_naive();
        let days_left = (due_date - now).num_days();
        let formatted = due_date.format("%b %d (%a)").to_string();
        (Some(days_left), formatted)
    } else {
        (None, "Invalid Date".to_string())
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
