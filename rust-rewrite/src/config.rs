use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AssignmentFolders {
    pub bib_folder: String,
    pub tex_folder: String,
    pub yaml_folder: String,
    pub pdf_folder: String,
    pub graded_assignment: String,
    pub online_assignment: String,
    pub solution_key: String,
}

#[derive(Debug, Deserialize)]
pub struct LessonManagerConfigFile {
    pub calendar_id: Option<String>,
    pub drive_folder_id: Option<String>,
    pub editor: String,
    pub terminal: String,
    pub pdf_viewer: String,
    pub create_readme_file: bool,
    pub highlight_current_course: bool,
    pub notes_dir: String,
    pub root: String,
    pub templates_dir: String,
    pub current_course: String,
    pub polybar_current_course_file: String,
    pub date_format: String,
    pub home: String,
    pub user: String,
    pub books_dir: String,
    pub figures_dir: String,
    pub assignments_dir: String,
    pub assignment_folders: AssignmentFolders,
    pub rofi_options: Vec<String>,
    pub folders: Vec<String>,
    pub files: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Professor {
    pub name: String,
    pub email: String,
    pub phone_number: String,
    pub office: String,
}

#[derive(Debug, Deserialize)]
pub struct CourseYamlFile {
    pub title: String,
    pub topic: String,
    pub class_number: u32,
    pub short: String,
    pub author: String,
    pub term: String,
    pub faculty: String,
    pub college: String,
    pub location: String,
    pub year: u32,
    pub start_date: String,
    pub end_date: String,
    pub start_time: String,
    pub end_time: String,
    pub days: String,
    pub url: String,
    pub professor: Professor,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Grade {
    Letter(String),
    Percentage(f64),
}

#[derive(Debug, Deserialize)]
pub struct AssignmentYamlFile {
    pub title: String,
    pub due_date: String,
    pub url: String,
    pub submitted: bool,
    pub grade: Option<Grade>,
    pub number: Option<u32>,
}
