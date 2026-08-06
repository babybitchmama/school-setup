use chrono::Local;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;

use crate::config::LessonManagerConfigFile;

use crate::open_in_neovim;

pub fn new_meeting(config: &LessonManagerConfigFile) {
    let thesis_dir = &config.thesis_dir;
    let terminal = &config.terminal;
    let editor = &config.editor;

    let thesis_path = PathBuf::from(shellexpand::tilde(thesis_dir).to_string());

    let today = Local::now().format("%m-%d-%Y").to_string();
    let meeting_dir = thesis_path.join("meetings").join(&today);

    if !meeting_dir.exists() {
        fs::create_dir_all(&meeting_dir).expect("Failed to create meeting directory");
    }

    let template_dir = thesis_path.join("meetings").join("templates");
    let target_files = ["notes", "questions", "solutions"];
    let mut files_to_open = Vec::new();

    for file_name in target_files.iter() {
        let template_path = template_dir.join(format!("{}-preamble.tex", file_name));
        let target_path = meeting_dir.join(format!("{}.tex", file_name));

        if !target_path.exists() {
            if template_path.exists() {
                fs::copy(&template_path, &target_path)
                    .unwrap_or_else(|_| panic!("Failed to copy template: {:?}", template_path));
            } else {
                fs::File::create(&target_path).expect("Failed to create empty file");
            }
        }

        files_to_open.push(target_path);
    }

    open_in_neovim(&meeting_dir, &files_to_open, terminal, editor, &config.editor_mode);
}

pub fn list_meetings(config: &LessonManagerConfigFile) {
    let thesis_dir = &config.thesis_dir;
    let terminal = &config.terminal;
    let editor = &config.editor;

    let thesis_path = PathBuf::from(shellexpand::tilde(thesis_dir).to_string());
    let meetings_base_dir = thesis_path.join("meetings");

    let Ok(entries) = fs::read_dir(&meetings_base_dir) else {
        message(
            "Failed to read meetings directory.",
            "info",
            &config.rofi_options,
            None,
        );
        return;
    };

    let mut meeting_dates = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                if folder_name != "templates" {
                    meeting_dates.push(folder_name.to_string());
                }
            }
        }
    }

    if meeting_dates.is_empty() {
        message("No meetings found.", "info", &config.rofi_options, None);
        return;
    }

    meeting_dates.sort_by(|a, b| b.cmp(a));

    if let Some(selected_date) = select_from_rofi(
        meeting_dates,
        &config.rofi_options,
        "Select a meeting date:".to_string(),
    ) {
        let selected_meeting_dir = meetings_base_dir.join(&selected_date);
        let pattern = format!("{}/*.tex", selected_meeting_dir.display());

        let mut file_display_list = Vec::new();
        let mut file_path_map = HashMap::new();

        if let Ok(file_entries) = glob::glob(&pattern) {
            for path in file_entries.flatten() {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let title_cased = stem
                        .split('-')
                        .map(|w| {
                            let mut chars = w.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");

                    file_display_list.push(title_cased.clone());
                    file_path_map.insert(title_cased, path);
                }
            }
        }

        if file_display_list.is_empty() {
            message(
                "No .tex files found in that meeting folder.",
                "info",
                &config.rofi_options,
                None,
            );
            return;
        }

        file_display_list.sort();

        if let Some(selected_file) = select_from_rofi(
            file_display_list,
            &config.rofi_options,
            "Select a file to open:".to_string(),
        ) {
            if let Some(target_path) = file_path_map.get(&selected_file) {
                open_in_neovim(
                    &selected_meeting_dir,
                    &[target_path.clone()],
                    terminal,
                    editor,
                    &config.editor_mode
                );
            }
        }
    }
}
