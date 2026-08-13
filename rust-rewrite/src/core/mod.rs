pub fn get_current_course_info(current_course_path: &str) -> crate::config::CourseYamlFile {
    let expanded_path = shellexpand::tilde(current_course_path);
    crate::yaml::load_file(expanded_path.as_ref())
        .expect("Failed to load current course info.yaml")
}

pub fn get_courses_in_path(root_dir_path: &str) -> Vec<String> {
    let expanded_path = shellexpand::tilde(root_dir_path);
    let Ok(paths) = std::fs::read_dir(expanded_path.as_ref()) else {
        return Vec::new();
    };

    let mut course_names: Vec<String> = Vec::new();

    for entry in paths.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                let info_path = entry.path().join("info.yaml");

                if info_path.exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        course_names.push(name.to_string());
                    }
                }
            }
        }
    }

    course_names
}

pub fn change_current_course(root_dir_path: &str, course_name: &str, notes_dir: &str) {
    let expanded_root = shellexpand::tilde(root_dir_path);
    let expanded_notes = shellexpand::tilde(notes_dir);

    let mut course_path = PathBuf::from(expanded_root.as_ref());
    course_path.push(course_name);

    let mut current_course_path = PathBuf::from(expanded_notes.as_ref());
    current_course_path.push("current-course");

    if current_course_path.exists() || current_course_path.is_symlink() {
        std::fs::remove_file(&current_course_path)
            .expect("Failed to remove existing current-course symlink");
    }

    std::os::unix::fs::symlink(&course_path, &current_course_path)
        .expect("Failed to create new symlink");
}

pub fn update_polybar_current_course(polybar_file: &str, course_name: &str) {
    let formatted_course_name = course_name.replace(" ", "-").to_uppercase();
    std::fs::write(polybar_file, formatted_course_name)
        .expect("Failed to write current course to polybar file");
}

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

// Imports
use std::path::PathBuf;

// Modules
pub mod assignments;
pub mod books;
pub mod calendar;
pub mod courses;
pub mod figures;
pub mod notes;
pub mod sync;
pub mod thesis;
