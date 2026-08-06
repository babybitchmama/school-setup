use chrono::Datelike;

pub fn get_week(date: &chrono::NaiveDate) -> u32 {
    let iso_week = date.iso_week();
    iso_week.week()
}

pub fn format_date_and_time(date: &chrono::NaiveDateTime) -> String {
    date.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_time(date: &chrono::NaiveDateTime) -> String {
    date.format("%H:%M:%S").to_string()
}
