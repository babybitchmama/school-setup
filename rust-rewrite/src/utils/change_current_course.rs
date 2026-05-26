use std::path::PathBuf;
use shellexpand;

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
