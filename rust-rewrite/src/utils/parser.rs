use chrono::Datelike;
use chrono::NaiveDateTime;

/// Pads a number with a leading zero (e.g., 1 -> "01")
pub fn pad_number(num: u32) -> String {
    format!("{:02}", num)
}

/// Extracts the ISO week number from a given date
pub fn get_week(date: NaiveDateTime) -> u32 {
    date.iso_week().week()
}

/// Parses a string like "1-3, 5, 7", "all", or "last" into a flat Vector of lecture numbers.
pub fn parse_range_string(arg: &str, all_numbers: &[u32]) -> Vec<u32> {
    let mut result = Vec::new();
    let arg = arg.trim();

    // 1. Handle special string specifications
    match arg {
        "all" => return all_numbers.to_vec(),
        "last" => {
            if let Some(&last) = all_numbers.last() {
                return vec![last];
            }
            return vec![];
        }
        "prev_last" => {
            let len = all_numbers.len();
            return if len >= 2 {
                all_numbers[len - 2..].to_vec()
            } else {
                all_numbers.to_vec()
            };
        }
        "prev" => {
            let len = all_numbers.len();
            return if len >= 1 {
                all_numbers[..len - 1].to_vec()
            } else {
                vec![]
            };
        }
        _ => {}
    }

    // 2. Parse comma-separated numbers and ranges
    let parts = arg.split(',');
    for part in parts {
        let part = part.trim();
        if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() == 2 {
                if let (Ok(start), Ok(end)) = (bounds[0].parse::<u32>(), bounds[1].parse::<u32>()) {
                    if start <= end {
                        result.extend(start..=end);
                    } else {
                        // Handle reverse ranges (e.g., "5-3")
                        let mut rev_range: Vec<u32> = (end..=start).collect();
                        rev_range.reverse();
                        result.extend(rev_range);
                    }
                }
            }
        } else if let Ok(num) = part.parse::<u32>() {
            result.push(num);
        }
    }

    result
}
