use crate::config::{
    Assignment, AssignmentFile, AssignmentFolders, AssignmentYaml, LessonManagerConfigFile,
};
use crate::parser::generate_short_title;
use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{Local, NaiveDate};

pub fn check_if_assignment_is_due(due_date_str: &str, submitted: bool) -> (Option<i64>, String) {
    const ASSIGNMENT_DATE_FORMAT: &str = "%m-%d-%y";

    if submitted {
        return (None, "Submitted".to_string());
    }

    if let Ok(due_date) = NaiveDate::parse_from_str(due_date_str, ASSIGNMENT_DATE_FORMAT) {
        let now: NaiveDate = Local::now().date_naive();
        let days_left = (due_date - now).num_days();
        let formatted = due_date.format("%b %d (%a)").to_string();
        (Some(days_left), formatted)
    } else {
        (None, "Invalid Date".to_string())
    }
}

impl Assignment {
    pub fn new(
        yaml_root: PathBuf,
        assignment_folders: &AssignmentFolders,
        current_course: &str,
        assignments_dir: &str,
    ) -> Self {
        let name = yaml_root.file_stem().unwrap().to_string_lossy().to_string();
        let mut file_paths = HashMap::new();
        let mut options = HashMap::new();

        for (key, folder_path) in assignment_folders {
            let new_key = key.replace("_folder", "");

            let full_folder = format!("{}/{}/{}", current_course, assignments_dir, folder_path);
            let expanded = shellexpand::tilde(&full_folder);
            let mut base_path = PathBuf::from(expanded.as_ref());
            base_path.push(&name);

            let pattern = format!("{}.*", base_path.display());
            let mut final_path = None;
            let mut exists = false;

            if let Ok(mut paths) = glob::glob(&pattern)
                && let Some(Ok(p)) = paths.next()
            {
                final_path = Some(p);
                exists = true;

                let display_name = new_key.replace('_', " ");
                let title_cased = format!("View {} File", display_name);
                options.insert(title_cased, new_key.clone());
            }

            file_paths.insert(
                new_key,
                AssignmentFile {
                    path: final_path,
                    exists,
                },
            );
        }

        let mut info = None;
        let mut formatted_due_date = "Unknown".to_string();
        let mut days_left = None;

        if let Some(yaml_file) = file_paths.get("yaml")
            && let Some(path) = &yaml_file.path
            && let Ok(contents) = fs::read_to_string(path)
        {
            match serde_yaml::from_str::<AssignmentYaml>(&contents) {
                Ok(parsed_yaml) => {
                    let (d_left, d_str) =
                        check_if_assignment_is_due(&parsed_yaml.due_date, parsed_yaml.submitted);

                    days_left = d_left;
                    formatted_due_date = generate_short_title(&d_str, 28);
                    info = Some(parsed_yaml);
                }
                Err(e) => {
                    println!("Failed to parse {:?}: {}", path, e);
                }
            }
        }

        Assignment {
            root: yaml_root,
            name,
            file_paths,
            options,
            info,
            formatted_due_date,
            days_left,
        }
    }

    /// Determines the correct viewer based on file extension
    pub fn parse_command(&self, cmd_key: &str, terminal: &str, editor: &str, pdf_viewer: &str) {
        let Some(file_info) = self.file_paths.get(cmd_key) else {
            println!("Error: Key '{}' not found in file paths", cmd_key);
            return;
        };

        let Some(path) = &file_info.path else {
            println!("Error: File does not exist for '{}'", cmd_key);
            return;
        };

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "pdf" {
                self.edit_pdf(path, pdf_viewer);
            } else {
                self.edit_text(path, terminal, editor);
            }
        }
    }

    fn edit_pdf(&self, path: &Path, pdf_viewer: &str) {
        let _ = Command::new(pdf_viewer)
            .arg(path)
            .spawn()
            .expect("Failed to launch PDF viewer")
            .wait();
    }

    fn edit_text(&self, path: &Path, terminal: &str, editor: &str) {
        let listen_location = "/tmp/nvim.pipe";
        let mut nvim_args = Vec::new();

        if Path::new(listen_location).exists() {
            nvim_args.push("--server");
            nvim_args.push(listen_location);
            nvim_args.push("--remote");
        } else {
            nvim_args.push("--listen");
            nvim_args.push(listen_location);
        }

        let _ = Command::new(terminal)
            .arg(editor)
            .args(nvim_args)
            .arg(path)
            .env("NVIM_MODE", "latex")
            .spawn()
            .expect("Failed to open terminal and editor")
            .wait();
    }
}

pub struct Assignments {
    pub items: Vec<Assignment>,
    #[allow(dead_code)]
    pub titles: Vec<String>,
}

impl Assignments {
    pub fn new(
        current_course: &String,
        assignments_dir: &String,
        assignment_folders: &AssignmentFolders,
    ) -> Self {
        let mut items = Vec::new();

        let yaml_folder_str = assignment_folders
            .get("yaml_folder")
            .expect("yaml_folder not found in assignment_folders");

        let pattern = format!(
            "{}/{}/{}/*.yaml",
            current_course, assignments_dir, yaml_folder_str
        );
        let expanded_pattern = shellexpand::tilde(&pattern);
        if let Ok(entries) = glob::glob(&expanded_pattern) {
            for entry in entries.flatten() {
                let assignment =
                    Assignment::new(entry, assignment_folders, current_course, assignments_dir);

                if assignment.info.is_some() {
                    items.push(assignment);
                }
            }
        }

        items.sort_by(|a, b| {
            let num_a = a.info.as_ref().map(|i| i.number).unwrap_or(0);
            let num_b = b.info.as_ref().map(|i| i.number).unwrap_or(0);
            num_a.cmp(&num_b)
        });

        let titles = items.iter().map(|a| a.name.clone()).collect();

        Assignments { items, titles }
    }
}

pub fn main(config: &LessonManagerConfigFile, current_course_boolean: bool) {
    let assignment_folders = &config.assignment_folders;
    let rofi_options = &config.rofi_options;
    let assignments_dir = &config.assignments_dir;
    let terminal = &config.terminal;
    let editor = &config.editor;
    let pdf_viewer = &config.pdf_viewer;
    let course = &config.current_course;

    if !current_course_boolean {}

    let mut all_assignments = Assignments::new(&course, assignments_dir, assignment_folders).items;

    if all_assignments.is_empty() {
        message("You don't have any assignments.", "info", rofi_options, None);
        return;
    }

    all_assignments.sort_by(|a, b| {
        let num_a = a.info.as_ref().map(|i| i.number).unwrap_or(0);
        let num_b = b.info.as_ref().map(|i| i.number).unwrap_or(0);
        num_b.cmp(&num_a)
    });

    let mut rofi_display_list = Vec::with_capacity(all_assignments.len());
    let mut assignment_map = HashMap::with_capacity(all_assignments.len());

    for assignment in all_assignments {
        let info = assignment.info.as_ref().unwrap();

        let number_str = info.number;
        let title = generate_short_title(&info.title, 20);
        let due_date = generate_short_title(&assignment.formatted_due_date, 15);

        let days_left_str = match assignment.days_left {
            Some(d) if d < 0 => "Overdue!".to_string(),
            Some(d) => format!("{} days left", d),
            None => "Submitted".to_string(),
        };
        let days_left_short = generate_short_title(&days_left_str, 16);

        let grade_str = match &info.grade {
            Some(g) => format!("({}%)", g),
            None => "(NA)".to_string(),
        };

        let display_str = format!(
            "<span><b>{num:>2}. {title:<20}</b>  <small><i>{due:<15}  {days:<16}  {grade:>7}</i></small></span>",
            num = number_str,
            title = title,
            due = due_date,
            days = days_left_short,
            grade = grade_str,
        );

        rofi_display_list.push(display_str.clone());

        assignment_map.insert(display_str.trim().to_string(), assignment.clone());
    }

    if let Some(selected_str) = select_from_rofi(
        rofi_display_list,
        rofi_options,
        "Select an Assignment".to_string(),
    ) {
        if let Some(selected_assignment) = assignment_map.get(&selected_str) {
            let mut command_display_list: Vec<String> =
                selected_assignment.options.keys().cloned().collect();

            command_display_list.sort();

            if let Some(selected_cmd_display) = select_from_rofi(
                command_display_list,
                rofi_options,
                "Select a Command".to_string(),
            ) && let Some(raw_cmd) = selected_assignment.options.get(&selected_cmd_display)
            {
                selected_assignment.parse_command(raw_cmd, terminal, editor, pdf_viewer);
            }
        } else {
            println!(
                "⚠️ CRITICAL ERROR: Rofi returned a string not found in the map: '{}'",
                selected_str
            );
        }
    }
}
