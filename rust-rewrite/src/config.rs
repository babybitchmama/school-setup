use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

pub type AssignmentFolders = HashMap<String, String>;

#[derive(Debug, Deserialize, Clone)]
pub struct NoteTypeConfig {
    pub path: String,
    pub style: String,
    pub naming: Option<String>,
    pub files: Option<Vec<String>>,
    pub folder: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LessonManagerConfigFile {
    pub calendar_id: Option<String>,
    pub drive_folder_id: Option<String>,
    pub onedrive_folder_id: Option<String>,
    pub editor: String,
    pub editor_mode: String,
    pub inkscape_mode: String,
    pub terminal_class_name: String,
    pub terminal: String,
    pub pdf_viewer: String,

    pub date_format: String,
    pub rofi_options: Vec<String>,

    pub notes_dir: String,
    pub root: String,
    pub templates_dir: String,
    pub current_course: String,
    pub polybar_current_course_file: String,

    pub books_folder: String,
    pub book_solution_folder: String,

    pub assignments_dir: String,
    pub assignments_root: String,
    pub assignment_folders: AssignmentFolders,

    pub thesis_dir: String,
    pub thesis_advisor_info_file: String,
    pub thesis_note_types: HashMap<String, NoteTypeConfig>,

    pub figures_dir: String,
    pub figure_template: Vec<String>,

    pub folders: Vec<String>,
    pub files: HashMap<String, String>,
}

impl LessonManagerConfigFile {
    pub fn load() -> Self {
        let path = shellexpand::tilde("~/.config/lesson-manager/config.yaml").into_owned();
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read config file {}: {}", path, e));
        serde_yaml::from_str(&contents).expect("Failed to parse lesson-manager.yaml")
    }
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

#[derive(Debug, Clone)]
pub struct AssignmentFile {
    pub path: Option<PathBuf>,
    pub exists: bool,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub root: PathBuf,
    pub name: String,
    pub file_paths: HashMap<String, AssignmentFile>,
    pub options: HashMap<String, String>,
    pub info: Option<AssignmentYaml>,
    pub formatted_due_date: String,
    pub days_left: Option<i64>,
}

fn deserialize_grade<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<serde_yaml::Value> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(serde_yaml::Value::String(s)) if s == "NA" => Ok(None),
        Some(serde_yaml::Value::String(s)) => Ok(Some(s)),
        Some(serde_yaml::Value::Number(n)) => Ok(Some(n.to_string())),
        Some(_) => Ok(None),
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AssignmentYaml {
    pub title: String,
    #[serde(deserialize_with = "deserialize_grade")]
    pub grade: Option<String>,
    pub submitted: bool,
    pub number: u32,
    pub due_date: String,
    pub url: Option<String>,
}

use indexmap::IndexMap;

#[derive(Debug, Deserialize)]
pub struct AdvisorInfo {
    #[serde(flatten)]
    pub fields: IndexMap<String, serde_yaml::Value>,
}
