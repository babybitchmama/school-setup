use shellexpand;

pub fn get_current_course_info(current_course_path: &str) -> crate::config::CourseYamlFile {
    let expanded_path = shellexpand::tilde(current_course_path);
    crate::utils::load_yaml_file::load_file(expanded_path.as_ref())
        .expect("Failed to load current course info.yaml")
}
