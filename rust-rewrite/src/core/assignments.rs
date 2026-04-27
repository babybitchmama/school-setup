use crate::config::AssignmentFolders;
use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;
use crate::utils::assignments::{check_if_assignment_is_due, generate_short_title};
use crate::utils::parser::pad_number;

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Mirrors the structure of your assignment .yaml files
#[derive(Deserialize, Debug, Clone)]
pub struct AssignmentYaml {
    pub title: String,
    pub grade: Option<String>,
    pub submitted: bool,
    pub number: u32,
    pub due_date: String,
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

impl Assignment {
    pub fn new(
        yaml_root: PathBuf,
        assignment_folders: &HashMap<String, String>,
        date_format: &str,
    ) -> Self {
        let name = yaml_root.file_stem().unwrap().to_string_lossy().to_string();
        let mut file_paths = HashMap::new();
        let mut options = HashMap::new();

        for (key, folder_path) in assignment_folders {
            let new_key = key.replace("_folder", "");
            let mut base_path = PathBuf::from(shellexpand::tilde(folder_path).as_ref());
            base_path.push(&name);

            let pattern = format!("{}.*", base_path.display());
            let mut final_path = None;
            let mut exists = false;

            if let Ok(mut paths) = glob::glob(&pattern) {
                if let Some(Ok(p)) = paths.next() {
                    final_path = Some(p);
                    exists = true;

                    let display_name = new_key.replace('_', " ");
                    let title_cased = format!("View {} File", display_name);
                    options.insert(title_cased, new_key.clone());
                }
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

        if let Some(yaml_file) = file_paths.get("yaml") {
            if let Some(path) = &yaml_file.path {
                if let Ok(contents) = fs::read_to_string(path) {
                    if let Ok(parsed_yaml) = serde_yaml::from_str::<AssignmentYaml>(&contents) {
                        let (d_left, d_str) = check_if_assignment_is_due(
                            &parsed_yaml.due_date,
                            parsed_yaml.submitted,
                            date_format,
                        );

                        days_left = d_left;
                        formatted_due_date = generate_short_title(&d_str, 28);
                        info = Some(parsed_yaml);
                    }
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

        if path.extension().and_then(|e| e.to_str()) == Some("pdf") {
            self.edit_pdf(path, pdf_viewer);
        } else {
            self.edit_text(path, terminal, editor);
        }
    }

    fn edit_pdf(&self, path: &Path, pdf_viewer: &str) {
        Command::new(pdf_viewer)
            .arg(path)
            .spawn()
            .expect("Failed to launch PDF viewer");
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

        Command::new(terminal)
            .arg(editor)
            .args(nvim_args)
            .arg(path)
            .spawn()
            .expect("Failed to open terminal and editor");
    }
}

pub struct Assignments {
    pub items: Vec<Assignment>,
    pub titles: Vec<String>,
}

impl Assignments {
    pub fn new(assignment_folders: &AssignmentFolders, date_format: &str) -> Self {
        let mut items = Vec::new();

        if let Some(yaml_folder_str) = assignment_folders.get("yaml_folder") {
            let expanded_path = shellexpand::tilde(yaml_folder_str);
            let pattern = format!("{}/*.yaml", expanded_path);

            if let Ok(entries) = glob::glob(&pattern) {
                for entry in entries.flatten() {
                    let assignment = Assignment::new(entry, assignment_folders, date_format);

                    if assignment.info.is_some() {
                        items.push(assignment);
                    }
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
pub fn main(
    assignment_folders: &AssignmentFolders,
    assignments_dir: &str,
    date_format: &str,
    rofi_options: &[String],
    terminal: &str,
    editor: &str,
    pdf_viewer: &str,
) {
    let mut all_assignments = Assignments::new(assignment_folders, date_format).items;

    if all_assignments.is_empty() {
        message("You don't have any assignments.", "info", rofi_options);
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

        let number_str = pad_number(info.number);
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

        let pad1 = "\u{00A0}".repeat(25usize.saturating_sub(title.len()));
        let pad2 = "\u{00A0}".repeat(15usize.saturating_sub(due_date.len()));
        let pad3 = "\u{00A0}".repeat(16usize.saturating_sub(days_left_short.len()));

        let pad_grade = "\u{00A0}".repeat(7usize.saturating_sub(grade_str.len()));

        let display_str = format!(
            "<b>{num}. {title}</b>{p1} <i><small>{due}</small></i>{p2} <i><small>{days}</small></i>{p3} <i><small>{p_grade}{grade}</small></i>",
            num = number_str,
            title = title,
            p1 = pad1,
            due = due_date,
            p2 = pad2,
            days = days_left_short,
            p3 = pad3,
            p_grade = pad_grade,
            grade = grade_str
        );

        rofi_display_list.push(display_str.clone());

        assignment_map.insert(display_str.trim().to_string(), assignment.clone());
    }

    if let Some(selected_str) = select_from_rofi(rofi_display_list, rofi_options) {
        if let Some(selected_assignment) = assignment_map.get(&selected_str) {
            let mut command_display_list: Vec<String> =
                selected_assignment.options.keys().cloned().collect();

            command_display_list.sort();

            if let Some(selected_cmd_display) = select_from_rofi(command_display_list, rofi_options)
            {
                if let Some(raw_cmd) = selected_assignment.options.get(&selected_cmd_display) {
                    let expanded_dir = shellexpand::tilde(assignments_dir);
                    if let Err(e) = env::set_current_dir(expanded_dir.as_ref()) {
                        println!(
                            "⚠️ CRITICAL ERROR: Failed to change to assignments directory: {}",
                            e
                        );
                        return;
                    }

                    selected_assignment.parse_command(raw_cmd, terminal, editor, pdf_viewer);
                }
            }
        } else {
            println!(
                "⚠️ CRITICAL ERROR: Rofi returned a string not found in the map: '{}'",
                selected_str
            );
        }
    }
}
