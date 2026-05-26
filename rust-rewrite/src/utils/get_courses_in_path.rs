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
