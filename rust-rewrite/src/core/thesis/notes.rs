use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::rofi::input::prompt_input;
use chrono::Local;

use crate::config::LessonManagerConfigFile;
use crate::open_in_neovim;
use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;

pub fn create_note(config: &LessonManagerConfigFile, note_type: &str) {
    let note_config = match config.thesis_note_types.get(note_type) {
        Some(cfg) => cfg,
        None => {
            message(
                &format!("Error: '{}' not defined in config.yaml", note_type),
                "error",
                &config.rofi_options,
                None,
            );
            return;
        }
    };

    let thesis_path = PathBuf::from(shellexpand::tilde(&config.thesis_dir).to_string());
    let target_dir = thesis_path.join(&note_config.path);

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).expect("Failed to create directory");
    }

    let pretty_type_name = note_type.replace('-', " ");
    if note_config.style == "single" {
        let naming_strategy = note_config.naming.as_deref().unwrap_or("prompt");
        let base_name = if naming_strategy == "prompt" {
            let prompt_text = format!("{} Name", pretty_type_name.to_uppercase());
            let Some(raw_input) = prompt_input(&prompt_text, &config.rofi_options) else {
                return;
            };
            raw_input.trim().to_lowercase()
        } else if let Some(format_str) = naming_strategy.strip_prefix("date:") {
            Local::now().format(format_str).to_string()
        } else {
            naming_strategy.to_string()
        };

        if base_name.is_empty() {
            return;
        }

        let file_path = target_dir.join(format!("{}.tex", base_name));
        if !file_path.exists() {
            fs::File::create(&file_path).expect("Failed to create file");
        }

        open_in_neovim(
            &target_dir,
            &[file_path],
            &config.terminal,
            &config.editor,
            &String::from("n"),
        );
    } else if note_config.style == "folder" {
        let mut folder_names = Vec::new();
        if let Ok(entries) = fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                        if folder_name != "templates" {
                            folder_names.push(folder_name.to_string());
                        }
                    }
                }
            }
        }
        folder_names.sort_by(|a, b| b.cmp(a));

        let create_new_opt = format!("➕ Create New {}", pretty_type_name);
        let mut display_list = vec![create_new_opt.clone()];
        display_list.extend(folder_names);

        let Some(selected_option) = select_from_rofi(
            display_list,
            &config.rofi_options,
            format!("Select {} folder:", pretty_type_name),
        ) else {
            return;
        };

        if selected_option == create_new_opt {
            let naming_strategy = note_config.naming.as_deref().unwrap_or("date:%m-%d-%Y");

            let base_name = if naming_strategy == "prompt" {
                let prompt_text = format!("New {} Name:", pretty_type_name.to_uppercase());
                let Some(raw_input) = prompt_input(&prompt_text, &config.rofi_options) else {
                    return;
                };
                raw_input.trim().to_lowercase().replace(' ', "-")
            } else if let Some(format_str) = naming_strategy.strip_prefix("date:") {
                Local::now().format(format_str).to_string()
            } else {
                naming_strategy.to_string()
            };

            if base_name.is_empty() {
                return;
            }

            let folder_dir = target_dir.join(&base_name);
            if !folder_dir.exists() {
                fs::create_dir_all(&folder_dir).expect("Failed to create folder");
            }

            let default_files = vec!["notes".to_string()];
            let files_to_create = note_config.files.as_ref().unwrap_or(&default_files);
            let mut files_to_open = Vec::new();

            for file_name in files_to_create {
                let time_formatted = Local::now().format(file_name).to_string();
                let final_file_name = time_formatted.replace("{base_name}", &base_name);
                let file_path = folder_dir.join(format!("{}.tex", final_file_name));

                if !file_path.exists() {
                    fs::File::create(&file_path).expect("Failed to create file");
                }
                files_to_open.push(file_path);
            }
            open_in_neovim(
                &folder_dir,
                &files_to_open,
                &config.terminal,
                &config.editor,
                &String::from("n"),
            );
        } else {
            let folder_dir = target_dir.join(&selected_option);
            let prompt_text = format!("New file in {}:", selected_option);

            let Some(raw_input) = prompt_input(&prompt_text, &config.rofi_options) else {
                return;
            };

            let clean_input = raw_input.trim().to_lowercase().replace(' ', "-");
            if clean_input.is_empty() {
                return;
            }

            let time_formatted = Local::now().format(&clean_input).to_string();
            let final_file_name = time_formatted.replace("{base_name}", &selected_option);

            let file_path = folder_dir.join(format!("{}.tex", final_file_name));

            if !file_path.exists() {
                fs::File::create(&file_path).expect("Failed to create file");
            }

            open_in_neovim(
                &folder_dir,
                &[file_path],
                &config.terminal,
                &config.editor,
                &String::from("n"),
            );
        }
    } else {
        message(
            &format!("Unknown style '{}' in config", note_config.style),
            "error",
            &config.rofi_options,
            None,
        );
    }
}

pub fn list_notes(config: &LessonManagerConfigFile, note_type: &str) {
    let note_config = match config.thesis_note_types.get(note_type) {
        Some(cfg) => cfg,
        None => {
            message(
                &format!("Error: '{}' not defined in config.yaml", note_type),
                "error",
                &config.rofi_options,
                None,
            );
            return;
        }
    };

    let thesis_path = PathBuf::from(shellexpand::tilde(&config.thesis_dir).to_string());
    let target_dir = thesis_path.join(&note_config.path);

    if !target_dir.exists() {
        message(
            &format!("Directory does not exist: {}", target_dir.display()),
            "error",
            &config.rofi_options,
            None,
        );
        return;
    }

    let prompt_title = note_type.replace('-', " ");

    if note_config.style == "single" {
        let pattern = format!("{}/*.tex", target_dir.display());
        let mut display_list = Vec::new();
        let mut path_map = HashMap::new();

        if let Ok(entries) = glob::glob(&pattern) {
            for path in entries.flatten() {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    display_list.push(stem.to_string());
                    path_map.insert(stem.to_string(), path);
                }
            }
        }

        if display_list.is_empty() {
            message("No notes found.", "info", &config.rofi_options, None);
            return;
        }

        display_list.sort();

        if let Some(selected) = select_from_rofi(
            display_list,
            &config.rofi_options,
            format!("Select {} note:", prompt_title),
        ) {
            if let Some(target_path) = path_map.get(&selected) {
                open_in_neovim(
                    &target_dir,
                    &[target_path.clone()],
                    &config.terminal,
                    &config.editor,
                    &String::from("n"),
                );
            }
        }
    } else if note_config.style == "folder" {
        let Ok(entries) = fs::read_dir(&target_dir) else {
            message(
                "Failed to read directory",
                "error",
                &config.rofi_options,
                None,
            );
            return;
        };

        let mut folder_names = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                    if folder_name != "templates" {
                        folder_names.push(folder_name.to_string());
                    }
                }
            }
        }

        if folder_names.is_empty() {
            message("No folders found.", "info", &config.rofi_options, None);
            return;
        }

        folder_names.sort_by(|a, b| b.cmp(a));

        if let Some(selected_folder) = select_from_rofi(
            folder_names,
            &config.rofi_options,
            format!("Select {} folder:", prompt_title),
        ) {
            let selected_dir = target_dir.join(&selected_folder);
            let pattern = format!("{}/*.tex", selected_dir.display());

            let mut file_display_list = Vec::new();
            let mut file_path_map = HashMap::new();

            if let Ok(file_entries) = glob::glob(&pattern) {
                for path in file_entries.flatten() {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        file_display_list.push(stem.to_string());
                        file_path_map.insert(stem.to_string(), path);
                    }
                }
            }

            if file_display_list.is_empty() {
                message(
                    "No files found in this folder.",
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
                "Select file:".to_string(),
            ) {
                if let Some(target_path) = file_path_map.get(&selected_file) {
                    open_in_neovim(
                        &selected_dir,
                        &[target_path.clone()],
                        &config.terminal,
                        &config.editor,
                        &String::from("n"),
                    );
                }
            }
        }
    }
}
