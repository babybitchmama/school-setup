pub fn format_course_name(
    course_name: &str,
    course_short_name: &str,
    global_max_len: usize,
) -> String {
    let padding_needed = global_max_len.saturating_sub(course_name.len()) + 4;

    let padding = "\u{00A0}".repeat(padding_needed);

    format!(
        "<b>{name}</b>{pad}<i><small>({short})</small></i>",
        name = course_name,
        pad = padding,
        short = course_short_name
    )
}
