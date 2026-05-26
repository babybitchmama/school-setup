pub fn update_polybar_current_course(polybar_file: &str, course_name: &str) {
    let formatted_course_name = course_name.replace(" ", "-").to_uppercase();
    std::fs::write(polybar_file, formatted_course_name)
        .expect("Failed to write current course to polybar file");
}
